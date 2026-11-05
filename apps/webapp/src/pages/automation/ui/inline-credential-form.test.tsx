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

const HEADER: Schemas.AuthSchemeResponse = {
	kind: 'http_header',
	label: 'HTTP header',
	fields: [
		{
			name: 'header_name',
			label: 'Header name',
			kind: 'Text',
			required: true,
			secret: false,
			expression: false,
			visible_when: null,
		},
		{
			name: 'header_value',
			label: 'Header value',
			kind: 'Text',
			required: true,
			secret: true,
			expression: false,
			visible_when: null,
		},
	],
}

const BEARER: Schemas.AuthSchemeResponse = {
	kind: 'bearer_token',
	label: 'Bearer token',
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

function renderForm(
	props: Partial<React.ComponentProps<typeof InlineCredentialForm>> = {},
) {
	const onSubmit = vi.fn()
	const onCancel = vi.fn()
	render(
		<InlineCredentialForm
			authSchemes={[SCHEME_A]}
			isPending={false}
			error={null}
			onSubmit={onSubmit}
			onCancel={onCancel}
			{...props}
		/>,
	)
	return { onSubmit, onCancel }
}

describe('InlineCredentialForm — supplied origin', () => {
	it('submits kind, trimmed name, origin and the scheme data', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm()

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

	it('does not submit an incomplete form and says what is missing, field by field', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ authSchemes: [SCHEME_B] })

		await user.type(screen.getByLabelText('User'), 'bob')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).not.toHaveBeenCalled()
		expect(
			screen.getByText('Donnez un nom à cette identification.'),
		).toBeDefined()
		expect(screen.getAllByText('Champ obligatoire.')).toHaveLength(1)
		expect(screen.getByLabelText('Pass').getAttribute('aria-invalid')).toBe(
			'true',
		)
	})

	it('keeps the submit button usable so that the errors can show up', () => {
		renderForm()

		expect(isDisabled(screen.getByRole('button', { name: 'Créer' }))).toBe(
			false,
		)
	})

	it('shows no error before the first attempt to submit', () => {
		renderForm({ authSchemes: [SCHEME_B] })

		expect(screen.queryByText('Champ obligatoire.')).toBeNull()
		expect(
			screen.queryByText('Donnez un nom à cette identification.'),
		).toBeNull()
	})

	it('masks a secret scheme field behind a password input', () => {
		renderForm()

		expect(screen.getByLabelText('Token').getAttribute('type')).toBe('password')
	})

	it('expects a whole number in a number field', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({
			authSchemes: [
				{
					kind: 'scheme_n',
					label: 'Scheme N',
					fields: [
						{
							name: 'port',
							label: 'Port',
							kind: 'Number',
							required: true,
							secret: false,
							expression: false,
							visible_when: null,
						},
					],
				},
			],
		})

		await user.type(screen.getByLabelText('Nom'), 'x')
		await user.type(screen.getByLabelText('Port'), '25a')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).not.toHaveBeenCalled()
		expect(screen.getByText('Nombre entier attendu.')).toBeDefined()
	})
})

describe('InlineCredentialForm — French wording of the types', () => {
	it('names the type and its fields in French and prefills the header name', () => {
		renderForm({ authSchemes: [HEADER] })

		expect(screen.getByRole('combobox', { name: 'Type' }).textContent).toBe(
			'Clé d’API dans un en-tête',
		)
		expect(
			(screen.getByLabelText('Nom de l’en-tête') as HTMLInputElement).value,
		).toBe('X-API-Key')
		expect(screen.getByLabelText('Valeur')).toBeDefined()
	})

	it('offers the types by their French name and loads the defaults of the chosen one', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ authSchemes: [BEARER, HEADER] })

		await user.click(screen.getByRole('combobox', { name: 'Type' }))
		expect(await screen.findByRole('option', { name: 'Bearer' })).toBeDefined()
		await user.click(
			screen.getByRole('option', { name: 'Clé d’API dans un en-tête' }),
		)

		expect(
			(screen.getByLabelText('Nom de l’en-tête') as HTMLInputElement).value,
		).toBe('X-API-Key')

		await user.type(screen.getByLabelText('Nom'), 'API tarifs')
		await user.type(screen.getByLabelText('Valeur'), 'sk-1')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'http_header',
			name: 'API tarifs',
			origin: 'supplied',
			data: { header_name: 'X-API-Key', header_value: 'sk-1' },
		})
	})

	it('drops the defaults of the type it leaves', async () => {
		const user = userEvent.setup()
		renderForm({ authSchemes: [HEADER, BEARER] })

		await user.click(screen.getByRole('combobox', { name: 'Type' }))
		await user.click(await screen.findByRole('option', { name: 'Bearer' }))

		expect(screen.queryByLabelText('Nom de l’en-tête')).toBeNull()
		expect(screen.getByLabelText('Jeton')).toBeDefined()
	})
})

describe('InlineCredentialForm — origin', () => {
	it('defaults to supplied and lets it switch to generated, which hides the scheme fields', async () => {
		const user = userEvent.setup()
		renderForm()

		expect(screen.getByLabelText('Token')).toBeDefined()

		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)

		expect(screen.queryByLabelText('Token')).toBeNull()
	})

	it('submits with origin generated and no data once the name is set', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm()

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
		renderForm({ lockOrigin: 'generated' })

		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()
		expect(screen.queryByLabelText('Token')).toBeNull()
	})

	it('submits origin generated without asking when locked', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ lockOrigin: 'generated' })

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
		renderForm({ error: 'La création a échoué' })

		expect(screen.getByText('La création a échoué')).toBeDefined()
	})

	it('shows a server conflict in French, without its prefix or constraint name', () => {
		renderForm({ error: 'Conflict: uq_automation_credential_name' })

		expect(
			screen.getByText('Une identification de ce type porte déjà ce nom.'),
		).toBeDefined()
		expect(screen.queryByText(/Conflict/)).toBeNull()
	})

	it('names the field the server found missing by its French label', () => {
		renderForm({
			authSchemes: [HEADER],
			error: 'Conflict: missing field `header_value`',
		})

		expect(
			screen.getByText('Le champ « Valeur » est obligatoire.'),
		).toBeDefined()
	})

	it('disables submit while pending and never submits', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ isPending: true })

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')

		const submit = screen.getByRole('button', {
			name: 'Créer',
		})
		expect(isDisabled(submit)).toBe(true)
		await user.click(submit)
		expect(onSubmit).not.toHaveBeenCalled()
	})

	it('calls onCancel', async () => {
		const user = userEvent.setup()
		const { onCancel } = renderForm()

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onCancel).toHaveBeenCalledTimes(1)
	})
})
