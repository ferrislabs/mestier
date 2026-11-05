import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	credentialFormDefaults,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import type {
	LocalizedField,
	LocalizedScheme,
} from '#/pages/automation/lib/credential-meta'
import {
	CredentialDialog,
	CredentialFieldRows,
} from '#/pages/automation/ui/credential-dialog'

for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

function field(
	name: string,
	overrides: Partial<LocalizedField> = {},
): LocalizedField {
	return {
		name,
		label: name,
		required: true,
		kind: 'Text',
		expression: false,
		secret: false,
		...overrides,
	}
}

const SSH_LIKE: LocalizedScheme = {
	kind: 'ssh_like',
	label: 'SSH',
	fields: [
		field('method', {
			label: 'Méthode',
			kind: {
				Select: {
					options: [
						{ value: 'password', label: 'Mot de passe' },
						{ value: 'key', label: 'Clé privée' },
					],
				},
			},
			defaultValue: 'password',
		}),
		field('password', {
			label: 'Secret par mot de passe',
			secret: true,
			visible_when: { field: 'method', any_of: ['password'] },
		}),
		field('private_key', {
			label: 'Secret par clé',
			secret: true,
			kind: 'Json',
			visible_when: { field: 'method', any_of: ['key'] },
		}),
	],
}

function Harness({
	scheme,
	fieldErrors = {},
}: {
	scheme: LocalizedScheme
	fieldErrors?: Record<string, string>
}) {
	const [values, setValues] = useState({
		...emptyCredentialForm(scheme.kind),
		name: 'Une identification',
		data: credentialFormDefaults(scheme),
	})
	return (
		<CredentialDialog
			open
			mode="create"
			editing={null}
			authSchemes={[scheme]}
			services={[]}
			form={{
				values,
				isPending: false,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				onSubmit: vi.fn(),
			}}
			fieldErrors={fieldErrors}
			saveError={null}
			revealedSecret={null}
			rotatingId={null}
			isDeleting={false}
			schemesLoading={false}
			onOpenChange={vi.fn()}
			onRotate={vi.fn()}
			onDelete={vi.fn()}
			onConnect={vi.fn()}
			onDisconnect={vi.fn()}
		/>
	)
}

describe('CredentialDialog — conditional fields', () => {
	it('shows only the fields the controlling field calls for, and follows its changes', async () => {
		const user = userEvent.setup()
		render(<Harness scheme={SSH_LIKE} />)

		expect(screen.getByLabelText('Secret par mot de passe')).toBeDefined()
		expect(screen.queryByLabelText('Secret par clé')).toBeNull()

		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		await user.click(await screen.findByRole('option', { name: 'Clé privée' }))

		expect(screen.getByLabelText('Secret par clé')).toBeDefined()
		expect(screen.queryByLabelText('Secret par mot de passe')).toBeNull()
	})
})

describe('CredentialDialog — hints', () => {
	const scopes = field('scopes', {
		label: 'Scopes',
		required: false,
		placeholder: 'openid profile',
		hint: 'Séparés par des espaces.',
	})
	const scheme: LocalizedScheme = { kind: 'k', label: 'K', fields: [scopes] }

	it('shows the placeholder and the hint under the field', () => {
		render(<Harness scheme={scheme} />)

		expect(screen.getByLabelText(/Scopes/)).toHaveProperty(
			'placeholder',
			'openid profile',
		)
		expect(screen.getByText('Séparés par des espaces.')).toBeDefined()
	})

	it('swaps the hint for the error once the form was submitted with one', async () => {
		const user = userEvent.setup()
		render(
			<Harness
				scheme={scheme}
				fieldErrors={{ scopes: 'Champ obligatoire.' }}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(screen.getByText('Champ obligatoire.')).toBeDefined()
		expect(screen.queryByText('Séparés par des espaces.')).toBeNull()
	})
})

describe('CredentialFieldRows — kinds of control', () => {
	function rows(fields: LocalizedField[], stored = false) {
		render(
			<CredentialFieldRows
				fields={fields}
				data={{}}
				baseId="t"
				stored={stored}
				errorOf={() => undefined}
				onChange={vi.fn()}
			/>,
		)
	}

	it('shows a multi-line field that is not secret as a readable text area', () => {
		rows([field('certificate', { label: 'Certificat', kind: 'Json' })])

		const control = screen.getByLabelText('Certificat')
		expect(control.tagName).toBe('TEXTAREA')
		expect(control.className).not.toContain('text-security')
	})

	it('masks a multi-line field that is secret', () => {
		rows([
			field('private_key', {
				label: 'Clé privée',
				kind: 'Json',
				secret: true,
			}),
		])

		const control = screen.getByLabelText('Clé privée')
		expect(control.tagName).toBe('TEXTAREA')
		expect(control.className).toContain('text-security')
	})

	it('says a stored value is unchanged instead of showing the placeholder', () => {
		rows(
			[field('host', { label: 'Hôte', placeholder: 'mail.exemple.fr' })],
			true,
		)

		expect(screen.getByLabelText('Hôte')).toHaveProperty(
			'placeholder',
			'Inchangé',
		)
	})

	it('marks an optional field as such, but not a choice', () => {
		rows([
			field('a', { label: 'Facultatif', required: false }),
			field('b', {
				label: 'Choix',
				required: false,
				kind: { Select: { options: [{ value: 'x', label: 'X' }] } },
			}),
		])

		expect(screen.getByText('(facultatif)')).toBeDefined()
		expect(screen.getAllByText(/facultatif/)).toHaveLength(1)
	})
})
