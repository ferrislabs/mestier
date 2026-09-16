import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { ConnectorConfigForm } from '#/pages/automation/ui/connector-config-form'

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

function baseProps(
	overrides: Partial<Parameters<typeof ConnectorConfigForm>[0]> = {},
) {
	return {
		descriptor: descriptor(),
		config: {},
		credentialId: null,
		credentials: [],
		errors: [],
		onConfigChange: vi.fn(),
		onCredentialChange: vi.fn(),
		onOpenExpression: vi.fn(),
		onRequestCreateCredential: vi.fn(),
		...overrides,
	}
}

describe('ConnectorConfigForm — rendering every kind', () => {
	it('renders one control per declared field, seeded from config', () => {
		const fields = [
			field({ name: 'url', label: 'URL', kind: 'Text' }),
			field({ name: 'timeout', label: 'Timeout', kind: 'Number' }),
			field({ name: 'active', label: 'Actif', kind: 'Bool' }),
			field({
				name: 'method',
				label: 'Méthode',
				kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
			}),
			field({ name: 'headers', label: 'Headers', kind: 'Json' }),
		]

		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ fields }),
					config: { url: 'https://a', timeout: 5, active: true },
				})}
			/>,
		)

		expect((screen.getByLabelText('URL') as HTMLInputElement).value).toBe(
			'https://a',
		)
		expect((screen.getByLabelText('Timeout') as HTMLInputElement).value).toBe(
			'5',
		)
		expect(screen.getByLabelText('Headers')).toBeDefined()
		expect(screen.getByRole('combobox', { name: 'Méthode' })).toBeDefined()
	})
})

describe('ConnectorConfigForm — visible_when', () => {
	it('hides the dependent field until the controlling one matches, and reveals it once it does', async () => {
		const user = userEvent.setup()
		const mode = field({
			name: 'mode',
			label: 'Mode',
			kind: {
				Select: {
					options: [
						{ value: 'simple', label: 'Simple' },
						{ value: 'advanced', label: 'Avancé' },
					],
				},
			},
		})
		const advancedOnly = field({
			name: 'raw',
			label: 'Brut',
			visible_when: { field: 'mode', any_of: ['advanced'] },
		})

		function Harness() {
			const [config, setConfig] = useState<Record<string, unknown>>({
				mode: 'simple',
			})
			return (
				<ConnectorConfigForm
					{...baseProps({
						descriptor: descriptor({ fields: [mode, advancedOnly] }),
						config,
						onConfigChange: setConfig,
					})}
				/>
			)
		}

		render(<Harness />)

		expect(screen.queryByLabelText('Brut')).toBeNull()

		await user.click(screen.getByRole('combobox', { name: 'Mode' }))
		await user.click(await screen.findByRole('option', { name: 'Avancé' }))

		expect(await screen.findByLabelText('Brut')).toBeDefined()
	})
})

describe('ConnectorConfigForm — editing a field', () => {
	it('reports the whole config, patched with the edited field', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL' })],
					}),
					config: { other: 'kept' },
					onConfigChange,
				})}
			/>,
		)

		fireEvent.change(screen.getByLabelText('URL'), {
			target: { value: 'https://x' },
		})

		expect(onConfigChange).toHaveBeenLastCalledWith({
			other: 'kept',
			url: 'https://x',
		})
	})
})

describe('ConnectorConfigForm — field-scoped validation errors', () => {
	it('lands each error message on the field it names', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({ name: 'url', label: 'URL' }),
							field({ name: 'method', label: 'Méthode' }),
						],
					}),
					errors: [
						{ field: 'url', message: 'URL manquante' },
						{ field: 'method', message: 'Méthode invalide' },
					],
				})}
			/>,
		)

		expect(screen.getByText('URL manquante')).toBeDefined()
		expect(screen.getByText('Méthode invalide')).toBeDefined()
	})
})

describe('ConnectorConfigForm — the typed credential slot', () => {
	it('offers only the credentials the AuthRequirement accepts, without naming a connector', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						auth: { AnyOf: ['bearer_token', 'http_basic', 'http_header'] },
					}),
					credentials: [
						credential('c1', 'bearer_token'),
						credential('c2', 'odoo_api'),
						credential('c3', 'http_basic'),
					],
					onCredentialChange,
				})}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))

		expect(screen.getByRole('option', { name: 'c1' })).toBeDefined()
		expect(screen.getByRole('option', { name: 'c3' })).toBeDefined()
		expect(screen.queryByRole('option', { name: 'c2' })).toBeNull()

		await user.click(screen.getByRole('option', { name: 'c1' }))
		expect(onCredentialChange).toHaveBeenCalledWith('c1')
	})

	it('renders no credential picker for a connector that needs none', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({ descriptor: descriptor({ auth: 'None' }) })}
			/>,
		)

		expect(
			screen.queryByRole('combobox', { name: 'Identification' }),
		).toBeNull()
	})

	it('asks to create a typed-slot credential through the request callback', async () => {
		const user = userEvent.setup()
		const onRequestCreateCredential = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'odoo_api' } }),
					onRequestCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /Créer|Nouvelle identification/ }),
		)

		expect(onRequestCreateCredential).toHaveBeenCalledWith('typed')
	})
})

describe('ConnectorConfigForm — expression assistant wiring', () => {
	it('hands each expression-enabled field its own control ref', () => {
		const onFieldRef = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					onFieldRef,
				})}
			/>,
		)

		expect(onFieldRef).toHaveBeenCalledWith('url', screen.getByLabelText('URL'))
	})

	it('reports a dropped path together with the field it landed on', () => {
		const onInsertExpression = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					onInsertExpression,
				})}
			/>,
		)

		const dataTransfer = { getData: () => 'trigger.id' }
		fireEvent.drop(screen.getByLabelText('URL'), { dataTransfer })

		expect(onInsertExpression).toHaveBeenCalledWith('url', 'trigger.id')
	})
})

describe('ConnectorConfigForm — the signing_credential_id field', () => {
	it('renders as a picker offering only generated-origin credentials, never a free-text input', async () => {
		const user = userEvent.setup()
		const onConfigChange = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'signing_credential_id',
								label: 'Signing credential',
								kind: 'Text',
							}),
						],
					}),
					credentials: [
						credential('c1', 'bearer_token', 'supplied'),
						credential('c2', 'bearer_token', 'generated'),
					],
					onConfigChange,
				})}
			/>,
		)

		expect(
			screen.queryByRole('textbox', { name: 'Signing credential' }),
		).toBeNull()

		await user.click(
			screen.getByRole('combobox', { name: 'Signing credential' }),
		)

		expect(screen.getByRole('option', { name: 'c2' })).toBeDefined()
		expect(screen.queryByRole('option', { name: 'c1' })).toBeNull()

		await user.click(screen.getByRole('option', { name: 'c2' }))

		expect(onConfigChange).toHaveBeenCalledWith({
			signing_credential_id: 'c2',
		})
	})

	it('asks to create a signing-slot credential through the request callback', async () => {
		const user = userEvent.setup()
		const onRequestCreateCredential = vi.fn()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'signing_credential_id',
								label: 'Signing credential',
							}),
						],
					}),
					onRequestCreateCredential,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /Créer|Nouvelle identification/ }),
		)

		expect(onRequestCreateCredential).toHaveBeenCalledWith('signing')
	})
})
