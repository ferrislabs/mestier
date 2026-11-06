import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { type ReactElement, useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { CredentialField, CredentialKind } from '#/hooks/use-automation'
import {
	credentialFormDefaults,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import { credentialRow } from '#/pages/automation/lib/credential-rows'
import {
	BEARER_TOKEN,
	credential,
	field,
	KINDS,
	OAUTH2_AUTHORIZATION_CODE,
	SSH,
} from '#/pages/automation/test/credential-fixtures'
import {
	CredentialDialog,
	CredentialFieldRows,
} from '#/pages/automation/ui/credential-dialog'
import { renderWithPermissions } from '#/test/with-permissions'

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

function renderManaged(ui: ReactElement) {
	return renderWithPermissions(ui, { permissions: ['MANAGE_AUTOMATION'] })
}

function Harness({
	descriptor,
	fieldErrors = {},
	serverErrors = {},
}: {
	descriptor: CredentialKind
	fieldErrors?: Record<string, string>
	serverErrors?: Record<string, string>
}) {
	const [values, setValues] = useState({
		...emptyCredentialForm(descriptor.kind),
		name: 'Une identification',
		data: credentialFormDefaults(descriptor),
	})
	return (
		<CredentialDialog
			open
			mode="create"
			editing={null}
			kinds={[descriptor]}
			services={[]}
			form={{
				values,
				isPending: false,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				onSubmit: vi.fn(),
			}}
			fieldErrors={fieldErrors}
			serverErrors={serverErrors}
			saveError={null}
			revealedSecret={null}
			rotatingId={null}
			isDeleting={false}
			isConnecting={false}
			kindsLoading={false}
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
		render(<Harness descriptor={SSH} />)

		expect(screen.getByLabelText('Clé privée')).toBeDefined()
		expect(screen.queryByLabelText('Mot de passe')).toBeNull()

		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		await user.click(
			await screen.findByRole('option', { name: 'Mot de passe' }),
		)

		expect(screen.getByLabelText('Mot de passe')).toBeDefined()
		expect(screen.queryByLabelText('Clé privée')).toBeNull()
	})
})

describe('CredentialDialog — what the descriptor says', () => {
	it('prefills the defaults and puts the placeholders and hints under the fields', () => {
		render(<Harness descriptor={OAUTH2_AUTHORIZATION_CODE} />)

		expect(screen.getByLabelText(/Scopes/)).toHaveProperty(
			'placeholder',
			'openid profile',
		)
		expect(screen.getByText('Séparés par des espaces.')).toBeDefined()
		expect(
			screen.getByRole('combobox', { name: 'PKCE' }).textContent,
		).toContain('Activé (recommandé)')
	})

	it('words the aside with the description and the usages of the kind', () => {
		render(<Harness descriptor={BEARER_TOKEN} />)

		const aside = screen.getByRole('complementary', { name: 'Type : Bearer' })
		expect(
			within(aside).getByText(
				'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
			),
		).toBeDefined()
		expect(
			within(aside).getByText(
				'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
			),
		).toBeDefined()
	})

	it('offers the origin only to a kind whose origin choices include a generated secret', () => {
		const { unmount } = render(<Harness descriptor={BEARER_TOKEN} />)
		expect(screen.getByRole('combobox', { name: 'Origine' })).toBeDefined()
		unmount()

		render(<Harness descriptor={SSH} />)
		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()
	})

	it('shows the server error under the field it names', () => {
		render(
			<Harness
				descriptor={OAUTH2_AUTHORIZATION_CODE}
				serverErrors={{
					client_secret:
						'Modifier cette adresse exige de saisir de nouveau « Client secret ».',
				}}
			/>,
		)

		expect(
			screen.getByText(
				'Modifier cette adresse exige de saisir de nouveau « Client secret ».',
			),
		).toBeDefined()
	})

	it('swaps the hint for the error once the form was submitted with one', async () => {
		const user = userEvent.setup()
		render(
			<Harness
				descriptor={OAUTH2_AUTHORIZATION_CODE}
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

describe('CredentialDialog — editing', () => {
	function editing(overrides: Partial<ReturnType<typeof credential>> = {}) {
		const row = credentialRow(
			credential({
				kind: 'oauth2_authorization_code',
				name: 'Agenda Google',
				used_by_workflows: 2,
				data: {
					authorization_url: 'https://a.fr/auth',
					token_url: 'https://a.fr/token',
					client_id: 'abc',
					pkce: 'yes',
				},
				secrets_set: ['client_secret'],
				...overrides,
			}),
			KINDS,
		)
		return {
			row,
			values: {
				...emptyCredentialForm(row.kind),
				name: row.name,
				data: { client_id: 'abc', pkce: 'yes' },
			},
		}
	}

	function renderEditing(
		overrides: Partial<ReturnType<typeof credential>> = {},
		handlers: { onConnect?: () => void; onDisconnect?: () => void } = {},
	) {
		const { row, values } = editing(overrides)
		renderManaged(
			<CredentialDialog
				open
				mode="edit"
				editing={row}
				kinds={[OAUTH2_AUTHORIZATION_CODE]}
				services={[]}
				form={{
					values,
					isPending: false,
					onChange: vi.fn(),
					onSubmit: vi.fn(),
				}}
				fieldErrors={{}}
				serverErrors={{}}
				saveError={null}
				revealedSecret={null}
				rotatingId={null}
				isDeleting={false}
				isConnecting={false}
				kindsLoading={false}
				onOpenChange={vi.fn()}
				onRotate={vi.fn()}
				onDelete={vi.fn()}
				onConnect={handlers.onConnect ?? vi.fn()}
				onDisconnect={handlers.onDisconnect ?? vi.fn()}
			/>,
		)
	}

	it('shows the secrets that are set as stored, and the others as plain inputs', () => {
		renderEditing()

		expect(
			screen.getByText('Enregistré et chiffré — non affichable'),
		).toBeDefined()
		expect(screen.getByLabelText('Client ID')).toHaveProperty('value', 'abc')
	})

	it('asks to connect a connectable kind that has no account yet', async () => {
		const onConnect = vi.fn()
		const user = userEvent.setup()
		renderEditing({ status: 'not_connected' }, { onConnect })

		const section = screen.getByRole('region', { name: 'Compte autorisé' })
		expect(within(section).getByText('Compte non connecté')).toBeDefined()
		expect(
			within(section).queryByRole('button', { name: 'Déconnecter' }),
		).toBeNull()

		await user.click(
			within(section).getByRole('button', { name: 'Connecter un compte' }),
		)

		expect(onConnect).toHaveBeenCalledOnce()
	})

	it('offers to disconnect, and to reconnect, a connected account', async () => {
		const onDisconnect = vi.fn()
		const user = userEvent.setup()
		renderEditing(
			{ status: 'valid', account: 'a@exemple.fr' },
			{ onDisconnect },
		)

		const section = screen.getByRole('region', { name: 'Compte autorisé' })
		expect(within(section).getByText(/Compte a@exemple.fr/)).toBeDefined()
		expect(
			within(section).getByRole('button', { name: 'Reconnecter' }),
		).toBeDefined()

		await user.click(
			within(section).getByRole('button', { name: 'Déconnecter' }),
		)

		expect(onDisconnect).toHaveBeenCalledOnce()
	})

	it('tells an expired authorization to be renewed', () => {
		renderEditing({ status: 'expired', account: 'a@exemple.fr' })

		expect(screen.getByText('Expirée')).toBeDefined()
		expect(
			screen.getByText(
				'L’autorisation a expiré. Reconnectez le compte pour la renouveler.',
			),
		).toBeDefined()
	})

	it('names the workflows that will fail when asked to delete a used credential', async () => {
		const user = userEvent.setup()
		renderEditing()

		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))

		expect(
			screen.getByText(/Utilisée par 2 workflows : leurs nœuds échoueront/),
		).toBeDefined()
	})

	it('has no authorization section for a kind that does not connect to an account', () => {
		const { row, values } = editing({ kind: 'bearer_token' })
		renderManaged(
			<CredentialDialog
				open
				mode="edit"
				editing={row}
				kinds={[BEARER_TOKEN]}
				services={[]}
				form={{
					values: { ...values, kind: 'bearer_token', data: {} },
					isPending: false,
					onChange: vi.fn(),
					onSubmit: vi.fn(),
				}}
				fieldErrors={{}}
				serverErrors={{}}
				saveError={null}
				revealedSecret={null}
				rotatingId={null}
				isDeleting={false}
				isConnecting={false}
				kindsLoading={false}
				onOpenChange={vi.fn()}
				onRotate={vi.fn()}
				onDelete={vi.fn()}
				onConnect={vi.fn()}
				onDisconnect={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('region', { name: 'Compte autorisé' })).toBeNull()
	})
})

describe('CredentialFieldRows — kinds of control', () => {
	function rows(
		fields: CredentialField[],
		data: Record<string, string> = {},
		storedSecrets: string[] = [],
	) {
		render(
			<CredentialFieldRows
				fields={fields}
				data={data}
				baseId="t"
				storedSecrets={storedSecrets}
				errorOf={() => undefined}
				onChange={vi.fn()}
			/>,
		)
	}

	it('shows a multi-line field that is not secret as a readable text area', () => {
		rows([field('certificate', 'Certificat', { kind: 'textarea' })])

		const control = screen.getByLabelText(/Certificat/)
		expect(control.tagName).toBe('TEXTAREA')
		expect(control.className).not.toContain('text-security')
	})

	it('masks a multi-line field that is secret', () => {
		rows([
			field('private_key', 'Clé privée', {
				kind: 'textarea',
				secret: true,
				required: true,
			}),
		])

		const control = screen.getByLabelText('Clé privée')
		expect(control.tagName).toBe('TEXTAREA')
		expect(control.className).toContain('text-security')
	})

	it('masks a JSON field that is secret', () => {
		rows([
			field('json', 'Définition (JSON)', {
				kind: 'json',
				secret: true,
				required: true,
			}),
		])

		expect(screen.getByLabelText('Définition (JSON)').className).toContain(
			'text-security',
		)
	})

	it('shows a password as a masked input', () => {
		rows([
			field('token', 'Jeton', {
				kind: 'password',
				secret: true,
				required: true,
			}),
		])

		expect(screen.getByLabelText('Jeton')).toHaveProperty('type', 'password')
	})

	it('says a secret that is set is stored, and offers to replace it', () => {
		rows(
			[
				field('token', 'Jeton', {
					kind: 'password',
					secret: true,
					required: true,
				}),
			],
			{},
			['token'],
		)

		expect(
			screen.getByText('Enregistré et chiffré — non affichable'),
		).toBeDefined()
		expect(screen.getByRole('button', { name: 'Remplacer' })).toBeDefined()
	})

	it('shows a secret that is not set as an input even when editing', () => {
		rows(
			[
				field('passphrase', 'Phrase de passe', {
					kind: 'password',
					secret: true,
				}),
			],
			{},
			['token'],
		)

		expect(screen.getByLabelText(/Phrase de passe/)).toHaveProperty(
			'type',
			'password',
		)
		expect(
			screen.queryByText('Enregistré et chiffré — non affichable'),
		).toBeNull()
	})

	it('prefills a readable value', () => {
		rows([field('host', 'Hôte', { required: true })], {
			host: 'smtp.exemple.fr',
		})

		expect(screen.getByLabelText('Hôte')).toHaveProperty(
			'value',
			'smtp.exemple.fr',
		)
	})

	it('shows a number as a numeric input', () => {
		rows([field('port', 'Port', { kind: 'number', required: true })], {
			port: '465',
		})

		const control = screen.getByLabelText('Port')
		expect(control).toHaveProperty('value', '465')
		expect(control.getAttribute('inputmode')).toBe('numeric')
	})

	it('shows a boolean as a switch following the value', () => {
		rows([field('ssl', 'SSL', { kind: 'boolean' })], { ssl: 'true' })

		expect(
			screen.getByRole('switch', { name: 'SSL' }).getAttribute('aria-checked'),
		).toBe('true')
	})

	it('marks an optional field as such, but not a choice or a switch', () => {
		rows([
			field('a', 'Facultatif'),
			field('b', 'Choix', {
				kind: 'select',
				options: [{ value: 'x', label: 'X' }],
			}),
			field('c', 'Interrupteur', { kind: 'boolean' }),
		])

		expect(screen.getByText('(facultatif)')).toBeDefined()
		expect(screen.getAllByText(/facultatif/)).toHaveLength(1)
	})

	it('offers the options of a select', async () => {
		const user = userEvent.setup()
		rows(
			[
				field('security', 'Sécurité', {
					kind: 'select',
					default: 'ssl',
					options: [
						{ value: 'ssl', label: 'SSL/TLS' },
						{ value: 'none', label: 'Aucune' },
					],
				}),
			],
			{ security: 'ssl' },
		)

		await user.click(screen.getByRole('combobox', { name: 'Sécurité' }))

		expect(await screen.findByRole('option', { name: 'Aucune' })).toBeDefined()
	})
})
