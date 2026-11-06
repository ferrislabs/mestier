import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ConnectorParametersFeature,
	type ConnectorParametersFeatureProps,
} from '#/pages/automation/feature/connector-parameters-feature'
import {
	BEARER_TOKEN,
	credential as fixture,
} from '#/pages/automation/test/credential-fixtures'

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
	status: Schemas.CredentialStatusResponse = 'valid',
): Schemas.CredentialResponse {
	return fixture({ id, kind, name: id, origin, status })
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
		credentialKinds: [BEARER_TOKEN],
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
		const created = credential('new-cred', 'bearer_token', 'supplied')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'irrelevant' })

		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					onCredentialChange,
					onCreateCredential,
				})}
			/>,
		)

		await openCreation(user)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			expect(onCredentialChange).toHaveBeenCalledWith('new-cred')
		})

		expect(onCreateCredential).toHaveBeenCalledWith({
			kind: 'bearer_token',
			name: 'Ma clé',
			origin: 'supplied',
			data: { token: 's3cr3t' },
		})
		expect(screen.queryByLabelText('Nom')).toBeNull()
	})

	it('shows the once-only secret before applying a generated credential', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		const created = credential('gen-cred', 'bearer_token', 'generated')
		const onCreateCredential = vi
			.fn()
			.mockResolvedValue({ ...created, secret: 'generated-secret' })

		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
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
		const created = credential('sign-cred', 'bearer_token', 'generated')
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
			.mockRejectedValue(
				Object.assign(
					new Error(
						'Conflict: Une identification de ce type porte déjà ce nom.',
					),
					{ status: 409 },
				),
			)

		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					onCreateCredential,
				})}
			/>,
		)

		await openCreation(user)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(
			await screen.findByText(
				'Une identification de ce type porte déjà ce nom.',
			),
		).toBeDefined()
		expect(screen.getByLabelText('Nom')).toBeDefined()
	})

	it('cancels the inline creation without applying a selection', async () => {
		const user = userEvent.setup()
		const onCredentialChange = vi.fn()
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
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

describe('ConnectorParametersFeature — typing an expression in a Number', () => {
	it('writes the expression into the field the descriptor lets take one', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'partner_id',
								label: 'Identifiant du partenaire',
								kind: 'Number',
								expression: true,
							}),
						],
					}),
					config: {},
					onConfigChange,
				})}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Identifiant du partenaire'), {
			target: { value: '{{ connectors.c1.output.id }}' },
		})

		expect(onConfigChange).toHaveBeenCalledWith({
			partner_id: '{{ connectors.c1.output.id }}',
		})
	})

	it('shows the expression a node already holds, and previews it', async () => {
		const onEvaluateExpression = vi.fn().mockResolvedValue(12)
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'partner_id',
								label: 'Identifiant du partenaire',
								kind: 'Number',
								expression: true,
							}),
						],
					}),
					config: { partner_id: '{{ connectors.c1.output.id }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		expect(
			(screen.getByLabelText('Identifiant du partenaire') as HTMLInputElement)
				.value,
		).toBe('{{ connectors.c1.output.id }}')
		await waitFor(() => {
			expect(screen.getByText('Aperçu : 12')).toBeDefined()
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

		await waitFor(
			() => {
				expect(
					screen.getByText('missing path: trigger.customer.id'),
				).toBeDefined()
			},
			{ timeout: 2000 },
		)
	})

	it('previews every expression field at once, without needing focus', async () => {
		const onEvaluateExpression = vi.fn(async (template: unknown) =>
			template === '{{ trigger.a }}' ? 'alpha' : 'beta',
		)
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({ name: 'url', label: 'URL', expression: true }),
							field({ name: 'name', label: 'Nom', expression: true }),
							field({ name: 'note', label: 'Note', expression: true }),
						],
					}),
					config: {
						url: '{{ trigger.a }}',
						name: '{{ trigger.b }}',
						note: 'sans expression',
					},
					onEvaluateExpression,
				})}
			/>,
		)

		await waitFor(
			() => {
				expect(screen.getByText('Aperçu : alpha')).toBeDefined()
				expect(screen.getByText('Aperçu : beta')).toBeDefined()
			},
			{ timeout: 2000 },
		)
		expect(onEvaluateExpression).toHaveBeenCalledTimes(2)
	})

	it('previews the expression a Number field holds', async () => {
		const onEvaluateExpression = vi.fn().mockResolvedValue(7)
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'amount',
								label: 'Montant',
								kind: 'Number',
								expression: true,
							}),
						],
					}),
					config: { amount: '{{ connectors.c1.output.total }}' },
					onEvaluateExpression,
				})}
			/>,
		)

		await waitFor(
			() => {
				expect(screen.getByText('Aperçu : 7')).toBeDefined()
			},
			{ timeout: 2000 },
		)
	})
})

describe('ConnectorParametersFeature — the defaults of a new node', () => {
	const httpDescriptor = () =>
		descriptor({
			kind: 'http.request',
			fields: [
				field({
					name: 'method',
					label: 'Méthode',
					kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
				}),
				field({ name: 'url', label: 'URL', expression: true }),
			],
		})

	it('writes the defaults the config lacks, once, when the node opens', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: httpDescriptor(),
					config: {},
					onConfigChange,
				})}
			/>,
		)

		expect(onConfigChange).toHaveBeenCalledTimes(1)
		expect(onConfigChange).toHaveBeenCalledWith({
			method: 'GET',
		})
	})

	it('leaves a configured node alone', () => {
		const onConfigChange = vi.fn()
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: httpDescriptor(),
					config: { method: 'POST', url: '' },
					onConfigChange,
				})}
			/>,
		)

		expect(onConfigChange).not.toHaveBeenCalled()
	})

	it('does not write again when the config comes back with its defaults', () => {
		const onConfigChange = vi.fn()
		const { rerender } = render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: httpDescriptor(),
					config: {},
					onConfigChange,
				})}
			/>,
		)

		rerender(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: httpDescriptor(),
					config: { method: 'GET' },
					onConfigChange,
				})}
			/>,
		)

		expect(onConfigChange).toHaveBeenCalledTimes(1)
	})
})

describe('ConnectorParametersFeature — the identification slot', () => {
	it('flags an expired identification from the status the server gives it', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					credentials: [
						credential('cred-expired', 'bearer_token', 'supplied', 'expired'),
					],
					credentialId: 'cred-expired',
				})}
			/>,
		)

		expect(
			screen.getByText(
				'Cette identification est expirée. Renouvelez-la depuis Identifications.',
			),
		).toBeDefined()
	})

	it('flags an identification whose account is not connected', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					credentials: [
						credential('cred-off', 'bearer_token', 'supplied', 'not_connected'),
					],
					credentialId: 'cred-off',
				})}
			/>,
		)

		expect(
			screen.getByText(
				'Cette identification n’est pas connectée. Connectez le compte depuis Identifications.',
			),
		).toBeDefined()
	})

	it('does not flag a valid identification', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					credentials: [credential('cred-ok', 'bearer_token')],
					credentialId: 'cred-ok',
				})}
			/>,
		)

		expect(screen.queryByText(/est expirée/)).toBeNull()
		expect(screen.queryByText(/n’est pas connectée/)).toBeNull()
	})

	it('shows the hint the node comes with', () => {
		render(
			<ConnectorParametersFeature
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'bearer_token' } }),
					credentialHint: 'La spec déclare : bearerAuth.',
				})}
			/>,
		)

		expect(screen.getByText('La spec déclare : bearerAuth.')).toBeDefined()
	})
})
