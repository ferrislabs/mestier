import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	BEARER_TOKEN,
	HTTP_BASIC,
	HTTP_HEADER,
	SMTP,
	SSH,
} from '#/pages/automation/test/credential-fixtures'
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

function renderForm(
	props: Partial<React.ComponentProps<typeof InlineCredentialForm>> = {},
) {
	const onSubmit = vi.fn()
	const onCancel = vi.fn()
	render(
		<InlineCredentialForm
			kinds={[BEARER_TOKEN]}
			isPending={false}
			error={null}
			onSubmit={onSubmit}
			onCancel={onCancel}
			{...props}
		/>,
	)
	return { onSubmit, onCancel }
}

function failure(message: string, status: number) {
	return Object.assign(new Error(message), { status })
}

describe('InlineCredentialForm — supplied origin', () => {
	it('submits kind, trimmed name, origin and the data of the kind', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm()

		await user.type(screen.getByLabelText('Nom'), '  Ma clé  ')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'bearer_token',
			name: 'Ma clé',
			origin: 'supplied',
			data: { token: 's3cr3t' },
		})
	})

	it('sends the description when there is one, and leaves it out otherwise', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm()

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText(/Description/), '  Pour Odoo  ')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'bearer_token',
			name: 'Ma clé',
			description: 'Pour Odoo',
			origin: 'supplied',
			data: { token: 's3cr3t' },
		})
	})

	it('does not submit an incomplete form and says what is missing, field by field', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ kinds: [HTTP_BASIC] })

		await user.type(screen.getByLabelText('Utilisateur'), 'bob')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).not.toHaveBeenCalled()
		expect(
			screen.getByText('Donnez un nom à cette identification.'),
		).toBeDefined()
		expect(screen.getAllByText('Champ obligatoire.')).toHaveLength(1)
		expect(
			screen.getByLabelText('Mot de passe').getAttribute('aria-invalid'),
		).toBe('true')
	})

	it('keeps the submit button usable so that the errors can show up', () => {
		renderForm()

		expect(isDisabled(screen.getByRole('button', { name: 'Créer' }))).toBe(
			false,
		)
	})

	it('shows no error before the first attempt to submit', () => {
		renderForm({ kinds: [HTTP_BASIC] })

		expect(screen.queryByText('Champ obligatoire.')).toBeNull()
		expect(
			screen.queryByText('Donnez un nom à cette identification.'),
		).toBeNull()
	})

	it('masks a secret field behind a password input', () => {
		renderForm()

		expect(screen.getByLabelText('Jeton').getAttribute('type')).toBe('password')
	})

	it('expects a whole number in a number field', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ kinds: [SMTP] })

		await user.type(screen.getByLabelText('Nom'), 'x')
		await user.type(screen.getByLabelText('Hôte'), 'smtp.exemple.fr')
		await user.clear(screen.getByLabelText('Port'))
		await user.type(screen.getByLabelText('Port'), '25a')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).not.toHaveBeenCalled()
		expect(screen.getByText('Nombre entier attendu.')).toBeDefined()
	})

	it('sends a number as a number', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ kinds: [SMTP] })

		await user.type(screen.getByLabelText('Nom'), 'Relais')
		await user.type(screen.getByLabelText('Hôte'), 'smtp.exemple.fr')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'smtp',
			name: 'Relais',
			origin: 'supplied',
			data: { host: 'smtp.exemple.fr', port: 465, security: 'ssl' },
		})
	})

	it('shows only the fields the choices call for', async () => {
		const user = userEvent.setup()
		renderForm({ kinds: [SSH] })

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

describe('InlineCredentialForm — the types the server describes', () => {
	it('names the type and its fields as the server does and prefills the header name', () => {
		renderForm({ kinds: [HTTP_HEADER] })

		expect(screen.getByRole('combobox', { name: 'Type' }).textContent).toBe(
			'Clé d’API dans un en-tête',
		)
		expect(
			(screen.getByLabelText('Nom de l’en-tête') as HTMLInputElement).value,
		).toBe('X-API-Key')
		expect(screen.getByLabelText('Valeur')).toBeDefined()
	})

	it('offers the types by their label and loads the defaults of the chosen one', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ kinds: [BEARER_TOKEN, HTTP_HEADER] })

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
		renderForm({ kinds: [HTTP_HEADER, BEARER_TOKEN] })

		await user.click(screen.getByRole('combobox', { name: 'Type' }))
		await user.click(await screen.findByRole('option', { name: 'Bearer' }))

		expect(screen.queryByLabelText('Nom de l’en-tête')).toBeNull()
		expect(screen.getByLabelText('Jeton')).toBeDefined()
	})
})

describe('InlineCredentialForm — origin', () => {
	it('defaults to supplied and lets it switch to generated, which hides the fields', async () => {
		const user = userEvent.setup()
		renderForm()

		expect(screen.getByLabelText('Jeton')).toBeDefined()

		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)

		expect(screen.queryByLabelText('Jeton')).toBeNull()
	})

	it('offers no origin to a kind the server does not let Mestier generate', () => {
		renderForm({ kinds: [HTTP_HEADER] })

		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()
	})

	it('goes back to supplied when the chosen type cannot be generated', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ kinds: [BEARER_TOKEN, HTTP_HEADER] })

		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(screen.getByRole('combobox', { name: 'Type' }))
		await user.click(
			await screen.findByRole('option', { name: 'Clé d’API dans un en-tête' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'API')
		await user.type(screen.getByLabelText('Valeur'), 'sk')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'http_header',
			name: 'API',
			origin: 'supplied',
			data: { header_name: 'X-API-Key', header_value: 'sk' },
		})
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
			kind: 'bearer_token',
			name: 'Signature',
			origin: 'generated',
			data: undefined,
		})
	})

	it('locks the origin to generated and hides the picker when lockOrigin is set', () => {
		renderForm({ lockOrigin: 'generated' })

		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()
		expect(screen.queryByLabelText('Jeton')).toBeNull()
	})

	it('submits origin generated without asking when locked', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ lockOrigin: 'generated' })

		await user.type(screen.getByLabelText('Nom'), 'Signature')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledWith({
			kind: 'bearer_token',
			name: 'Signature',
			origin: 'generated',
			data: undefined,
		})
	})
})

describe('InlineCredentialForm — feedback', () => {
	it('shows the refusal of the server without its English prefix', () => {
		renderForm({
			error: failure(
				'Validation failed: JSON invalide : un objet est attendu.',
				400,
			),
		})

		expect(
			screen.getByText('JSON invalide : un objet est attendu.'),
		).toBeDefined()
		expect(screen.queryByText(/Validation failed/)).toBeNull()
	})

	it('shows a name already taken under the name', () => {
		renderForm({
			error: failure(
				'Conflict: Une identification de ce type porte déjà ce nom.',
				409,
			),
		})

		const name = screen.getByLabelText('Nom')
		expect(name.getAttribute('aria-invalid')).toBe('true')
		expect(
			screen.getByText('Une identification de ce type porte déjà ce nom.'),
		).toBeDefined()
	})

	it('puts the refusal that names a field label under that field', () => {
		renderForm({
			kinds: [HTTP_HEADER],
			error: failure(
				'Validation failed: Le champ « Valeur » est obligatoire.',
				400,
			),
		})

		expect(screen.getByLabelText('Valeur').getAttribute('aria-invalid')).toBe(
			'true',
		)
		expect(
			screen.getByText('Le champ « Valeur » est obligatoire.'),
		).toBeDefined()
	})

	it('disables submit while pending and never submits', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderForm({ isPending: true })

		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')

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
