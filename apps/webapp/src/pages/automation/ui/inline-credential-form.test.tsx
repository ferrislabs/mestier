import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { InlineCredentialForm } from '#/pages/automation/ui/inline-credential-form'

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

function isDisabled(element: HTMLElement): boolean {
	return (element as HTMLButtonElement).disabled
}

const SCHEME_A: Schemas.AuthSchemeResponse = {
	kind: 'scheme_a',
	label: 'Scheme A',
	fields: [
		{
			name: 'token',
			label: 'Token',
			kind: 'Text',
			required: true,
			secret: true,
			expression: false,
			visible_when: null,
		},
	],
}

const SCHEME_B: Schemas.AuthSchemeResponse = {
	kind: 'scheme_b',
	label: 'Scheme B',
	fields: [
		{
			name: 'user',
			label: 'User',
			kind: 'Text',
			required: true,
			secret: false,
			expression: false,
			visible_when: null,
		},
		{
			name: 'pass',
			label: 'Pass',
			kind: 'Text',
			required: true,
			secret: true,
			expression: false,
			visible_when: null,
		},
	],
}

describe('InlineCredentialForm — supplied origin', () => {
	it('keeps the submit button disabled until the name, kind and required data are filled', async () => {
		const user = userEvent.setup()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		const submit = screen.getByRole('button', { name: 'Créer' })
		expect(isDisabled(submit)).toBe(true)

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		expect(isDisabled(submit)).toBe(true)

		await user.type(screen.getByLabelText('Token'), 's3cr3t')
		expect(isDisabled(submit)).toBe(false)
	})

	it('submits kind, trimmed name, origin and the scheme data', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={onSubmit}
				onCancel={vi.fn()}
			/>,
		)

		await user.type(screen.getByLabelText('Nom'), '  Ma clé  ')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'scheme_a',
			name: 'Ma clé',
			origin: 'supplied',
			data: { token: 's3cr3t' },
		})
	})

	it('requires every field the chosen scheme declares', async () => {
		const user = userEvent.setup()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_B]}
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('User'), 'bob')

		expect(isDisabled(screen.getByRole('button', { name: 'Créer' }))).toBe(true)

		await user.type(screen.getByLabelText('Pass'), 'hunter2')

		expect(isDisabled(screen.getByRole('button', { name: 'Créer' }))).toBe(
			false,
		)
	})

	it('masks a secret scheme field behind a password input', () => {
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Token').getAttribute('type')).toBe('password')
	})
})

describe('InlineCredentialForm — origin', () => {
	it('defaults to supplied and lets it switch to generated, which hides the scheme fields', async () => {
		const user = userEvent.setup()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Token')).toBeDefined()

		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)

		expect(screen.queryByLabelText('Token')).toBeNull()
	})

	it('submits with origin generated and no data once name and kind are set', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={onSubmit}
				onCancel={vi.fn()}
			/>,
		)

		await user.type(screen.getByLabelText('Nom'), 'Signature')
		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'scheme_a',
			name: 'Signature',
			origin: 'generated',
			data: undefined,
		})
	})

	it('locks the origin to generated and hides the picker when lockOrigin is set', () => {
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				lockOrigin="generated"
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()
		expect(screen.queryByLabelText('Token')).toBeNull()
	})

	it('submits origin generated without asking when locked', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				lockOrigin="generated"
				isPending={false}
				error={null}
				onSubmit={onSubmit}
				onCancel={vi.fn()}
			/>,
		)

		await user.type(screen.getByLabelText('Nom'), 'Signature')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'scheme_a',
			name: 'Signature',
			origin: 'generated',
			data: undefined,
		})
	})
})

describe('InlineCredentialForm — feedback', () => {
	it('shows the given error', () => {
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error="La création a échoué"
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getByText('La création a échoué')).toBeDefined()
	})

	it('disables submit while pending', async () => {
		const user = userEvent.setup()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={true}
				error={null}
				onSubmit={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')

		expect(isDisabled(screen.getByRole('button', { name: 'Créer' }))).toBe(true)
	})

	it('calls onCancel', async () => {
		const user = userEvent.setup()
		const onCancel = vi.fn()
		render(
			<InlineCredentialForm
				authSchemes={[SCHEME_A]}
				isPending={false}
				error={null}
				onSubmit={vi.fn()}
				onCancel={onCancel}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onCancel).toHaveBeenCalledTimes(1)
	})
})
