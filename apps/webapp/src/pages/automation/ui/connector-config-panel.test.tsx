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
		connectorId: 'c1',
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
		onCreateCredential: vi.fn(),
		availableData: {
			tree: [],
			context: { trigger: null, connectors: {}, loop: null },
		},
		lastStep: null,
		onEvaluateExpression: vi.fn().mockResolvedValue(null),
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
		expect(screen.getByText('Dernière exécution')).toBeDefined()

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

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
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

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
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
			screen.getByRole('combobox', { name: 'Signing credential' }),
		)
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
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

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
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

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
		)
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByLabelText('Nom')).toBeNull()
		expect(onCredentialChange).not.toHaveBeenCalled()
	})
})

const TRIGGER_BRANCH = {
	kind: 'branch' as const,
	key: 'trigger',
	label: 'trigger',
	path: 'trigger',
	children: [
		{
			kind: 'leaf' as const,
			key: 'name',
			label: 'name',
			path: 'trigger.name',
			value: 'Julie',
		},
	],
}

describe('ConnectorConfigPanel — the available-data tree', () => {
	it('shows the tree open by default, fed from the given branches', () => {
		render(
			<ConnectorConfigPanel
				{...baseProps({
					availableData: {
						tree: [TRIGGER_BRANCH],
						context: { trigger: { name: 'Julie' }, connectors: {}, loop: null },
					},
				})}
			/>,
		)

		expect(screen.getByRole('button', { name: /trigger/ })).toBeDefined()
	})
})

describe('ConnectorConfigPanel — inserting an expression', () => {
	function setCursor(input: HTMLInputElement, position: number) {
		input.setSelectionRange(position, position)
	}

	it('inserts the clicked path at the cursor of the field last opened for expression editing', async () => {
		const user = userEvent.setup()
		const onConfigChange = vi.fn()
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: 'Hello ' },
					onConfigChange,
					availableData: {
						tree: [TRIGGER_BRANCH],
						context: { trigger: { name: 'Julie' }, connectors: {}, loop: null },
					},
				})}
			/>,
		)

		const input = screen.getByLabelText('URL') as HTMLInputElement
		setCursor(input, 6)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une donnée dans URL' }),
		)
		await user.click(screen.getByRole('button', { name: /trigger/ }))
		await user.click(
			screen.getByRole('button', { name: /Insérer trigger\.name/ }),
		)

		expect(onConfigChange).toHaveBeenCalledWith({
			url: 'Hello {{ trigger.name }}',
		})
	})

	it('inserts a dropped path on the field it was dropped on, without opening it first', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '' },
					onConfigChange,
				})}
			/>,
		)

		const dataTransfer = { getData: () => 'trigger.name' }
		fireEvent.drop(screen.getByLabelText('URL'), { dataTransfer })

		expect(onConfigChange).toHaveBeenCalledWith({
			url: '{{ trigger.name }}',
		})
	})
})

describe('ConnectorConfigPanel — the live preview', () => {
	it('shows the resolved value once the debounced evaluation returns', async () => {
		const onEvaluateExpression = vi.fn().mockResolvedValue({ id: 42 })
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '{{ connectors.c1.output.id }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: 'Insérer une donnée dans URL' }),
		)

		await waitFor(
			() => {
				expect(screen.getByText('{"id":42}')).toBeDefined()
			},
			{ timeout: 2000 },
		)

		expect(onEvaluateExpression).toHaveBeenCalledWith(
			'{{ connectors.c1.output.id }}',
			{ trigger: null, connectors: {}, loop: null },
		)
	})

	it('names the missing path verbatim, never a silent null', async () => {
		const onEvaluateExpression = vi
			.fn()
			.mockRejectedValue(new Error('missing path: trigger.customer.id'))
		render(
			<ConnectorConfigPanel
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '{{ trigger.customer.id }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: 'Insérer une donnée dans URL' }),
		)

		await waitFor(
			() => {
				expect(
					screen.getByText('missing path: trigger.customer.id'),
				).toBeDefined()
			},
			{ timeout: 2000 },
		)
	})
})

describe('ConnectorConfigPanel — switching connectors', () => {
	it('drops the active field when a different connector opens', () => {
		const { rerender } = render(
			<ConnectorConfigPanel
				{...baseProps({
					connectorId: 'c1',
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '' },
				})}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: 'Insérer une donnée dans URL' }),
		)
		expect(screen.getByText(/Aperçu/)).toBeDefined()

		rerender(
			<ConnectorConfigPanel
				{...baseProps({
					connectorId: 'c2',
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '' },
				})}
			/>,
		)

		expect(screen.queryByText(/Aperçu/)).toBeNull()
	})
})

describe('ConnectorConfigPanel — what the last run did to this connector', () => {
	function step(
		overrides: Partial<Schemas.RunStepResponse>,
	): Schemas.RunStepResponse {
		return {
			attempts: 1,
			connector_id: 'c1',
			created_at: '2026-09-17T10:00:00Z',
			id: 'step-1',
			iteration_path: '',
			status: 'succeeded',
			...overrides,
		}
	}

	it('says so when the connector has never run', () => {
		render(<ConnectorConfigPanel {...baseProps({ lastStep: null })} />)

		expect(screen.getByText('Aucune exécution.')).toBeDefined()
	})

	it('shows the error when the step failed, instead of claiming nothing ran', () => {
		render(
			<ConnectorConfigPanel
				{...baseProps({
					lastStep: step({ status: 'failed', error: 'connection refused' }),
				})}
			/>,
		)

		expect(screen.getByRole('alert').textContent).toContain(
			'connection refused',
		)
		expect(screen.queryByText('Aucune exécution.')).toBeNull()
	})

	it('shows the output when the step succeeded', () => {
		render(
			<ConnectorConfigPanel
				{...baseProps({
					lastStep: step({ status: 'succeeded', output: { id: 42 } }),
				})}
			/>,
		)

		expect(screen.getByText(/"id": 42/)).toBeDefined()
	})
})
