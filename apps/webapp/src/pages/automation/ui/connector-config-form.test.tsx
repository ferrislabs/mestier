import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { credential as fixture } from '#/pages/automation/test/credential-fixtures'
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
		execution: 'per_item',
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
		expect(screen.getByText('Headers')).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Ajouter un champ' }),
		).toBeDefined()
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

	it('offers only the smtp credentials to the e-mail node, the identification first', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'smtp.send',
						family: 'smtp',
						auth: { Exactly: 'smtp' },
						fields: [field({ name: 'to', label: 'Destinataire' })],
					}),
					credentials: [
						credential('relais', 'smtp'),
						credential('jeton', 'bearer_token'),
						credential('secours', 'smtp'),
					],
				})}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))

		expect(screen.getByRole('option', { name: 'relais' })).toBeDefined()
		expect(screen.getByRole('option', { name: 'secours' })).toBeDefined()
		expect(screen.queryByRole('option', { name: 'jeton' })).toBeNull()
		const picker = screen.getByText('Identification')
		expect(
			picker.compareDocumentPosition(screen.getByText('Destinataire')) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
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

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
		)

		expect(onRequestCreateCredential).toHaveBeenCalledWith('typed')
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
			screen.getByRole('combobox', { name: 'Signing credential' }),
		)
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
		)

		expect(onRequestCreateCredential).toHaveBeenCalledWith('signing')
	})
})

describe('ConnectorConfigForm — a generated action', () => {
	it('renders every field kind with the server label and no crash', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'mestier.invoice.send',
						family: 'mestier',
						fields: [
							field({ name: 'a', label: 'Texte serveur', kind: 'Text' }),
							field({ name: 'b', label: 'Nombre serveur', kind: 'Number' }),
							field({ name: 'c', label: 'Booléen serveur', kind: 'Bool' }),
							field({ name: 'd', label: 'Json serveur', kind: 'Json' }),
							field({
								name: 'e',
								label: 'Choix serveur',
								kind: {
									Select: { options: [{ value: 'x', label: 'Option X' }] },
								},
							}),
						],
					}),
				})}
			/>,
		)

		for (const label of [
			'Texte serveur',
			'Nombre serveur',
			'Booléen serveur',
			'Json serveur',
			'Choix serveur',
		]) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})
})

describe('ConnectorConfigForm — the copy of a known connector', () => {
	const httpFields = [
		field({
			name: 'method',
			label: 'Méthode',
			kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
		}),
		field({ name: 'url', label: 'URL', expression: true }),
		field({ name: 'body', label: 'Corps', kind: 'Json', expression: true }),
	]

	it('relabels the field the descriptor names too tersely', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ kind: 'http.request', fields: httpFields }),
				})}
			/>,
		)

		expect(screen.getByText('Corps de la requête')).toBeDefined()
		expect(screen.queryByText('Corps')).toBeNull()
	})

	it('leaves a connector the table does not know as the descriptor wrote it', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ fields: httpFields }),
				})}
			/>,
		)

		expect(screen.getByText('Corps')).toBeDefined()
	})

	it('shows a hint and a placeholder from the table', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'flow.condition',
						fields: [
							field({
								name: 'predicate',
								label: 'Condition',
								expression: true,
							}),
						],
					}),
				})}
			/>,
		)

		expect(
			screen.getByText(
				'Le flux continue par « vrai » si la condition est remplie, par « faux » sinon.',
			),
		).toBeDefined()
		expect(screen.getByLabelText('Condition').getAttribute('placeholder')).toBe(
			'Glissez une variable depuis la colonne Entrée',
		)
	})
})

describe('ConnectorConfigForm — the lot 7 nodes', () => {
	const mergeFields = [
		field({
			name: 'mode',
			label: 'Mode',
			required: true,
			kind: {
				Select: {
					options: [
						{ value: 'append', label: 'Ajouter les éléments à la suite' },
						{ value: 'combine_by_position', label: 'Combiner par position' },
						{ value: 'combine_by_field', label: 'Combiner par champ' },
					],
				},
			},
		}),
		field({
			name: 'field',
			label: 'Champ de jointure',
			visible_when: { field: 'mode', any_of: ['combine_by_field'] },
		}),
	]

	it('shows the join field of a merge only when it combines by field', () => {
		const merge = descriptor({
			kind: 'flow.merge',
			family: 'flow',
			fields: mergeFields,
		})
		const { rerender } = render(
			<ConnectorConfigForm
				{...baseProps({ descriptor: merge, config: { mode: 'append' } })}
			/>,
		)

		expect(screen.queryByLabelText('Champ de jointure')).toBeNull()

		rerender(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: merge,
					config: { mode: 'combine_by_field' },
				})}
			/>,
		)

		expect(screen.getByLabelText('Champ de jointure')).toBeDefined()
	})

	it('shows the duration of a wait as a number, the seeded value included', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'flow.wait',
						family: 'flow',
						fields: [
							field({
								name: 'duration',
								label: 'Durée',
								kind: 'Number',
								expression: true,
								required: true,
							}),
						],
					}),
					config: { duration: 2 },
				})}
			/>,
		)

		expect((screen.getByLabelText('Durée') as HTMLInputElement).value).toBe('2')
	})

	it('draws the message of a channel post and the text of a note as text areas', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'mestier.chat.post_message',
						family: 'mestier',
						fields: [
							field({ name: 'channel', label: 'Canal', expression: true }),
							field({ name: 'message', label: 'Message', expression: true }),
						],
					}),
					config: { channel: '#general', message: 'Nouveau devis à valider.' },
				})}
			/>,
		)

		expect(screen.getByLabelText('Canal').tagName).toBe('INPUT')
		expect(screen.getByLabelText('Message').tagName).toBe('TEXTAREA')
		expect((screen.getByLabelText('Message') as HTMLTextAreaElement).rows).toBe(
			4,
		)
	})
})

describe('ConnectorConfigForm — where the identification sits', () => {
	const fields = [
		field({
			name: 'method',
			label: 'Méthode',
			kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
		}),
		field({ name: 'url', label: 'URL' }),
		field({ name: 'body', label: 'Corps', kind: 'Json' }),
	]

	function order(): string[] {
		return ['Méthode', 'URL', 'Identification', 'Corps de la requête'].sort(
			(a, b) => {
				const first = screen.getAllByText(a)[0]
				const second = screen.getAllByText(b)[0]
				return first.compareDocumentPosition(second) &
					Node.DOCUMENT_POSITION_FOLLOWING
					? -1
					: 1
			},
		)
	}

	it('comes after the URL of an HTTP request, before its body', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'http.request',
						auth: { Optional: ['bearer_token'] },
						fields,
					}),
				})}
			/>,
		)

		expect(order()).toEqual([
			'Méthode',
			'URL',
			'Identification',
			'Corps de la requête',
		])
	})

	it('stays first for any other connector', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						kind: 'odoo.create_partner',
						auth: { Exactly: 'odoo_api' },
						fields: [field({ name: 'name', label: 'Nom' })],
					}),
				})}
			/>,
		)

		const picker = screen.getByText('Identification')
		const name = screen.getByText('Nom')
		expect(
			picker.compareDocumentPosition(name) & Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})
})

describe('ConnectorConfigForm — the identification slot', () => {
	it('shows the hint under the picker', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'odoo_api' } }),
					credentialHint: 'La spec déclare : bearerAuth.',
				})}
			/>,
		)

		expect(screen.getByText('La spec déclare : bearerAuth.')).toBeDefined()
	})

	it('flags an expired identification, in the list and under the picker', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({ auth: { Exactly: 'odoo_api' } }),
					credentials: [credential('c1', 'odoo_api', 'supplied', 'expired')],
					credentialId: 'c1',
				})}
			/>,
		)

		expect(
			screen.getByText(
				'Cette identification est expirée. Renouvelez-la depuis Identifications.',
			),
		).toBeDefined()

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		expect(screen.getByRole('option', { name: 'c1 (expirée)' })).toBeDefined()
	})
})

describe('ConnectorConfigForm — the preview of an expression', () => {
	it('asks for the preview of each field, and shows it under an expression', () => {
		const renderPreview = vi.fn((named: Schemas.FieldResponse) => (
			<p>Aperçu de {named.name}</p>
		))
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({ name: 'url', label: 'URL', expression: true }),
							field({ name: 'name', label: 'Nom', expression: true }),
						],
					}),
					config: { url: '{{ trigger.id }}', name: 'Atelier' },
					renderPreview,
				})}
			/>,
		)

		expect(screen.getByText('Aperçu de url')).toBeDefined()
		expect(screen.queryByText('Aperçu de name')).toBeNull()
	})
})

describe('ConnectorConfigForm — what the caller adds to a field', () => {
	it('shows the placeholder and the hint a field is given beyond the descriptor', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({ name: 'path.id', label: 'id', expression: true }),
							field({ name: 'query.q', label: 'q', expression: true }),
						],
					}),
					fieldAddOns: {
						'path.id': {
							placeholder: 'Obligatoire',
							hint: 'Chemin · obligatoire',
						},
					},
				})}
			/>,
		)

		const required = screen.getByLabelText('id') as HTMLInputElement
		expect(required.placeholder).toBe('Obligatoire')
		expect(screen.getByText('Chemin · obligatoire')).toBeDefined()
		expect((screen.getByLabelText('q') as HTMLInputElement).placeholder).toBe(
			'',
		)
	})

	it('types the keys of a JSON field after the kinds it is given', () => {
		render(
			<ConnectorConfigForm
				{...baseProps({
					descriptor: descriptor({
						fields: [
							field({
								name: 'body',
								label: 'Corps',
								kind: 'Json',
								expression: true,
							}),
						],
					}),
					config: { body: { age: null } },
					fieldAddOns: { body: { entryKinds: { age: 'number' } } },
				})}
			/>,
		)

		expect(screen.getByLabelText('Type de age').textContent).toBe('Nombre')
	})
})
