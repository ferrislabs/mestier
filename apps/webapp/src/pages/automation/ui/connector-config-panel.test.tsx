import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ConnectorConfigPanel,
	type ConnectorConfigPanelProps,
	MAX_PANEL_WIDTH,
	MIN_PANEL_WIDTH,
} from '#/pages/automation/ui/connector-config-panel'

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
	overrides: Partial<Schemas.FieldResponse> = {},
): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: overrides.name ?? 'Field',
		name: 'field',
		required: false,
		secret: false,
		visible_when: null,
		...overrides,
	}
}

function descriptor(
	overrides: Partial<Schemas.ConnectorDescriptorResponse> = {},
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family: 'test',
		fields: [],
		kind: 'test.fabricated',
		label: 'Fabriqué',
		output_example: null,
		version: 1,
		...overrides,
	}
}

function credential(
	id: string,
	kind: string,
	origin: 'supplied' | 'generated' = 'supplied',
): Schemas.CredentialResponse {
	return {
		id,
		kind,
		name: id,
		origin,
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
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

function baseProps(
	overrides: Partial<ConnectorConfigPanelProps> = {},
): ConnectorConfigPanelProps {
	return {
		label: 'Requête HTTP',
		descriptor: descriptor(),
		config: {},
		credentialId: null,
		credentials: [],
		authSchemes: [SCHEME_A],
		errors: [],
		onClose: vi.fn(),
		onConfigChange: vi.fn(),
		onCredentialChange: vi.fn(),
		onOpenExpression: vi.fn(),
		onCreateCredential: vi.fn(),
		...overrides,
	}
}

describe('ConnectorConfigPanel — layout', () => {
	it('shows the connector label, the three zones, and closes on request', async () => {
		const user = userEvent.setup()
		const onClose = vi.fn()
		render(<ConnectorConfigPanel {...baseProps({ onClose })} />)

		expect(screen.getByText('Requête HTTP')).toBeDefined()
		expect(screen.getByText('Données disponibles')).toBeDefined()
		expect(screen.getByText('Paramètres')).toBeDefined()
		expect(screen.getByText('Dernière sortie')).toBeDefined()

		await user.click(screen.getByRole('button', { name: /Fermer/ }))
		expect(onClose).toHaveBeenCalledTimes(1)
	})

	it('renders the fields of the descriptor inside the parameters zone', () => {
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL' })],
					}),
					config: { url: 'https://a' },
				})}
			/>,
		)

		expect((screen.getByLabelText('URL') as HTMLInputElement).value).toBe(
			'https://a',
		)
	})

	it('forwards a field edit to onConfigChange', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL' })],
					}),
					onConfigChange,
				})}
			/>,
		)

		fireEvent.change(screen.getByLabelText('URL'), {
			target: { value: 'https://x' },
		})

		expect(onConfigChange).toHaveBeenCalledWith({ url: 'https://x' })
	})

	it('banners a connector-level error rather than showing it as a toast', () => {
		render(
			<ConnectorConfigPanel
				{...baseProps({
					errors: [
						{ field: null, message: 'Identifiant de credential manquant' },
					],
				})}
			/>,
		)

		expect(screen.getByText('Identifiant de credential manquant')).toBeDefined()
	})
})

describe('ConnectorConfigPanel — resizing', () => {
	it('widens when the handle is dragged toward the left edge of the window', () => {
		render(<ConnectorConfigPanel {...baseProps()} />)

		const panel = screen.getByTestId('connector-config-panel')
		const initialWidth = Number.parseInt(panel.style.width, 10)

		const handle = screen.getByRole('button', { name: /Redimensionner/ })
		fireEvent.mouseDown(handle, { clientX: 500 })
		fireEvent.mouseMove(window, { clientX: 400 })
		fireEvent.mouseUp(window)

		expect(Number.parseInt(panel.style.width, 10)).toBe(initialWidth + 100)
	})

	it('clamps the width between the panel bounds', () => {
		render(<ConnectorConfigPanel {...baseProps()} />)

		const panel = screen.getByTestId('connector-config-panel')
		const handle = screen.getByRole('button', { name: /Redimensionner/ })

		fireEvent.mouseDown(handle, { clientX: 500 })
		fireEvent.mouseMove(window, { clientX: -10_000 })
		fireEvent.mouseUp(window)
		expect(Number.parseInt(panel.style.width, 10)).toBe(MAX_PANEL_WIDTH)

		fireEvent.mouseDown(handle, { clientX: 500 })
		fireEvent.mouseMove(window, { clientX: 10_000 })
		fireEvent.mouseUp(window)
		expect(Number.parseInt(panel.style.width, 10)).toBe(MIN_PANEL_WIDTH)
	})
})

describe('ConnectorConfigPanel — inline credential creation', () => {
	it('creates a supplied credential, selects it, and returns to the form without a secret step', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		const created = credential('new-cred', 'scheme_a', 'supplied')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'irrelevant' })

		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
					onCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			expect(onCredentialChange).toHaveBeenCalledWith('new-cred')
		})

		expect(onCreateCredential).toHaveBeenCalledWith({
			kind: 'scheme_a',
			name: 'Ma clé',
			origin: 'supplied',
			data: { token: 's3cr3t' },
		})
		expect(screen.queryByLabelText('Nom')).toBeNull()
	})

	it('shows the once-only secret before applying a generated credential', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		const created = credential('gen-cred', 'scheme_a', 'generated')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'generated-secret' })

		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
					onCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Signature')
		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			await screen.findByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		const secretInput = (await screen.findByLabelText(
			'Secret',
		)) as HTMLInputElement
		expect(secretInput.value).toBe('generated-secret')
		expect(onCredentialChange).not.toHaveBeenCalled()

		await user.click(screen.getByRole('button', { name: 'Continuer' }))

		expect(onCredentialChange).toHaveBeenCalledWith('gen-cred')
	})

	it('routes a signing-slot creation into config, locked to a generated origin', async () => {
		const user = userEvent.setup()
		const onConfigChange = vi.fn()
		const created = credential('sign-cred', 'scheme_a', 'generated')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'signing-secret' })

		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'signing_credential_id',
								label: 'Signing credential',
							}),
						],
					}),
					onConfigChange,
					onCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)

		expect(screen.queryByRole('combobox', { name: 'Origine' })).toBeNull()

		await user.type(screen.getByLabelText('Nom'), 'Signature')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		const secretInput = (await screen.findByLabelText(
			'Secret',
		)) as HTMLInputElement
		expect(secretInput.value).toBe('signing-secret')
		await user.click(screen.getByRole('button', { name: 'Continuer' }))

		expect(onConfigChange).toHaveBeenCalledWith({
			signing_credential_id: 'sign-cred',
		})
	})

	it('shows the creation error and keeps the form open', async () => {
		const user = userEvent.setup()
		const onCreateCredential = vi
			.fn()
			.mockRejectedValue(new Error('La création a échoué'))

		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(await screen.findByText('La création a échoué')).toBeDefined()
		expect(screen.getByLabelText('Nom')).toBeDefined()
	})

	it('cancels the inline creation without applying a selection', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByLabelText('Nom')).toBeNull()
		expect(onCredentialChange).not.toHaveBeenCalled()
	})
})
