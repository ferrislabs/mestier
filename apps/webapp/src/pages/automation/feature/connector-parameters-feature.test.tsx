import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ConnectorParametersFeature,
	type ConnectorParametersFeatureProps,
} from '#/pages/automation/feature/connector-parameters-feature'

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

const EMPTY_CONTEXT: Schemas.EvaluateContextBody = {
	trigger: null,
	connectors: {},
	loop: null,
}

function baseProps(
	overrides: Partial<ConnectorParametersFeatureProps> = {},
): ConnectorParametersFeatureProps {
	return {
		descriptor: descriptor(),
		config: {},
		credentialId: null,
		credentials: [],
		authSchemes: [SCHEME_A],
		errors: [],
		bindingIssues: [],
		evaluateContext: EMPTY_CONTEXT,
		onConfigChange: vi.fn(),
		onCredentialChange: vi.fn(),
		onCreateCredential: vi.fn(),
		onEvaluateExpression: vi.fn().mockResolvedValue(null),
		...overrides,
	}
}

describe('ConnectorParametersFeature — the fields', () => {
	it('renders the fields of the descriptor', () => {
		render(
			<ConnectorParametersFeature
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
			<ConnectorParametersFeature
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

	it('banners a connector-level error on a connector with no credential', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					errors: [
						{ field: null, message: 'Identifiant de credential manquant' },
					],
				})}
			/>,
		)

		expect(screen.getByRole('alert').textContent).toContain(
			'Identifiant de credential manquant',
		)
	})

	it('names a field-scoped error and a broken reference under that field', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '{{ connectors.c9.output.id }}' },
					errors: [{ field: 'url', message: 'URL invalide' }],
					bindingIssues: [
						{ parameter: 'url', message: 'Le nœud « c9 » n’existe plus.' },
					],
				})}
			/>,
		)

		const alerts = screen
			.getAllByRole('alert')
			.map((alert) => alert.textContent)
		expect(alerts).toEqual(['URL invalide', 'Le nœud « c9 » n’existe plus.'])
		expect(screen.getByText('Expression')).toBeDefined()
	})
})

describe('ConnectorParametersFeature — inline credential creation', () => {
	async function openCreation(user: ReturnType<typeof userEvent.setup>) {
		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
		)
	}

	it('creates a supplied credential, selects it, and returns to the form without a secret step', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		const created = credential('new-cred', 'scheme_a', 'supplied')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'irrelevant' })

		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
					onCreateCredential,
				})}
			/>,
		)

		await openCreation(user)
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
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
					onCreateCredential,
				})}
			/>,
		)

		await openCreation(user)
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
			<ConnectorParametersFeature
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
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCreateCredential,
				})}
			/>,
		)

		await openCreation(user)
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
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'scheme_a' } }),
					onCredentialChange,
				})}
			/>,
		)

		await openCreation(user)
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByLabelText('Nom')).toBeNull()
		expect(onCredentialChange).not.toHaveBeenCalled()
	})
})

describe('ConnectorParametersFeature — inserting an expression', () => {
	it('inserts a dropped path at the cursor of the field it was dropped on', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: 'Hello ' },
					onConfigChange,
				})}
			/>,
		)

		const input = screen.getByLabelText('URL') as HTMLInputElement
		input.setSelectionRange(6, 6)
		fireEvent.drop(input, { dataTransfer: { getData: () => 'trigger.name' } })

		expect(onConfigChange).toHaveBeenCalledWith({
			url: 'Hello {{ trigger.name }}',
		})
	})
})

describe('ConnectorParametersFeature — the live preview', () => {
	it('shows the resolved value once the debounced evaluation returns', async () => {
		const onEvaluateExpression = vi.fn().mockResolvedValue({ id: 42 })
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '{{ connectors.c1.output.id }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		fireEvent.focus(screen.getByLabelText('URL'))

		await waitFor(
			() => {
				expect(screen.getByText('Aperçu : {"id":42}')).toBeDefined()
			},
			{ timeout: 2000 },
		)

		expect(onEvaluateExpression).toHaveBeenCalledWith(
			'{{ connectors.c1.output.id }}',
			EMPTY_CONTEXT,
		)
	})

	it('names the missing path verbatim, never a silent null', async () => {
		const onEvaluateExpression = vi
			.fn()
			.mockRejectedValue(new Error('missing path: trigger.customer.id'))
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [field({ name: 'url', label: 'URL', expression: true })],
					}),
					config: { url: '{{ trigger.customer.id }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		fireEvent.focus(screen.getByLabelText('URL'))

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
