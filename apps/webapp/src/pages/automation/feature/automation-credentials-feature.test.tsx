import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { AutomationCredentialsFeature } from '#/pages/automation/feature/automation-credentials-feature'
import {
	credentialMetaOf,
	patchCredentialMeta,
} from '#/pages/automation/feature/mocks/credential-meta'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

/**
 * jsdom has no pointer-capture APIs or `scrollIntoView`. Radix's `Select`
 * calls both when an item is actually *selected* — opening the popover
 * alone never reaches this path, which is why no other test in this repo
 * needed it before this file, the first to click a `SelectItem` rather
 * than just asserting it exists.
 *
 * Deliberately scoped to this file rather than `vitest.setup.ts`: filling
 * the gap globally changes real (if accidental) behavior every other test
 * that opens a Radix `Select`/`Popover` was already running against —
 * tried once, reverted after it broke 24 unrelated tests across 9 files.
 */
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

const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const CREDENTIAL_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}'
const CREDENTIAL_ROTATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const AUTH_SCHEMES = [
	{
		kind: 'bearer_token',
		label: 'Bearer token',
		fields: [
			{
				name: 'token',
				label: 'Token',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	},
	{
		kind: 'http_header',
		label: 'HTTP header',
		fields: [
			{
				name: 'header_name',
				label: 'Header name',
				required: true,
				kind: 'Text',
				expression: false,
				secret: false,
			},
			{
				name: 'header_value',
				label: 'Header value',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	},
	{
		kind: 'odoo_api',
		label: 'Odoo API key',
		fields: [
			{
				name: 'base_url',
				label: 'Instance URL',
				required: true,
				kind: 'Text',
				expression: false,
				secret: false,
			},
			{
				name: 'api_key',
				label: 'API key',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	},
]

function credential(overrides: Record<string, unknown> = {}) {
	return {
		id: 'cred-1',
		organization_id: 'org-1',
		name: 'Signature sortante',
		kind: 'bearer_token',
		origin: 'generated',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

type Handler = (params: unknown) => unknown

interface FakeApi {
	calls: { method: string; path: string; params: unknown }[]
	mockGet: (path: string, handler: Handler) => void
	mockMutation: (method: string, path: string, handler: Handler) => void
}

function installFakeTanstackApi(): FakeApi {
	const calls: { method: string; path: string; params: unknown }[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						const handler = mutationHandlers.get(`${method}:${path}`)
						if (!handler) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return handler(params)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		calls,
		mockGet: (path, handler) => getHandlers.set(path, handler),
		mockMutation: (method, path, handler) =>
			mutationHandlers.set(`${method}:${path}`, handler),
	}
}

function renderFeature(
	configure: (api: FakeApi) => void,
	permissions: string[] = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
) {
	const api = installFakeTanstackApi()
	api.mockGet(CONNECTORS_PATH, () => ({
		data: { connectors: [], auth_schemes: AUTH_SCHEMES },
	}))
	api.mockGet(CREDENTIALS_PATH, () => ({ data: [] }))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	render(
		<Providers>
			<AutomationCredentialsFeature />
		</Providers>,
	)

	return api
}

describe('AutomationCredentialsFeature — creation', () => {
	it('reveals a generated secret exactly once after creation', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: credential({ secret: 'c2VjcmV0LWJ5dGVz' }),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))
		await user.type(screen.getByLabelText('Nom'), 'Signature sortante')
		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			screen.getByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('c2VjcmV0LWJ5dGVz')).toBeDefined()
	})

	it('never shows a secret reveal for a supplied credential', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, (params) => ({
				data: credential({
					origin: 'supplied',
					secret: (params as { body: { data: unknown } }).body.data,
				}),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))
		await user.type(screen.getByLabelText('Nom'), 'Odoo')
		await user.type(screen.getByLabelText('Jeton'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		await waitFor(() => {
			expect(screen.queryByText('Secret généré')).toBeNull()
		})
	})

	it('requires every scheme field before submitting a supplied credential', async () => {
		const user = userEvent.setup()
		const api = renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))
		await user.type(screen.getByLabelText('Nom'), 'Sans token')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(screen.getAllByText('Champ obligatoire.').length).toBeGreaterThan(0)
		expect(
			api.calls.some((c) => c.method === 'post' && c.path === CREDENTIALS_PATH),
		).toBe(false)
	})
})

describe('AutomationCredentialsFeature — rotation', () => {
	it('reveals the freshly rotated secret exactly once', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('post', CREDENTIAL_ROTATE_PATH, () => ({
				data: credential({ secret: 'cm90YXRlZC1zZWNyZXQ' }),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(
			await screen.findByRole('button', { name: 'Régénérer le secret' }),
		)

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('cm90YXRlZC1zZWNyZXQ')).toBeDefined()
	})

	it('offers no rotation on a supplied credential', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ origin: 'supplied' })],
			}))
		})

		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await screen.findByRole('dialog')
		expect(
			screen.queryByRole('button', { name: 'Régénérer le secret' }),
		).toBeNull()
	})
})

describe('AutomationCredentialsFeature — deletion refused', () => {
	it('names the workflows the backend reports when a referenced credential cannot be deleted', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('delete', CREDENTIAL_PATH, () => {
				throw new Error(
					'Conflict: credential is still referenced by workflow(s): Créer une facture Odoo',
				)
			})
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		expect(
			await screen.findByText(
				'Impossible de la supprimer : elle est encore utilisée par Créer une facture Odoo.',
			),
		).toBeDefined()
	})
})

describe('AutomationCredentialsFeature — permission gating', () => {
	it('hides every mutation without MANAGE_AUTOMATION', async () => {
		renderFeature(
			(api) => {
				api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			},
			['VIEW_AUTOMATION'],
		)

		await screen.findByText('Signature sortante')
		expect(
			screen.queryByRole('button', { name: 'Nouvelle identification' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Signature sortante' }),
		).toBeNull()
	})
})

describe('AutomationCredentialsFeature — mocked status and usage', () => {
	it('shows the usage and status columns with the mocked values', async () => {
		patchCredentialMeta('cred-meta', {
			usedByWorkflows: 2,
			status: 'expired',
			description: 'Clé du fournisseur de tarifs.',
		})
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ id: 'cred-meta', origin: 'supplied' })],
			}))
		})

		expect(await screen.findByText('Utilisée par 2 workflows')).toBeDefined()
		expect(screen.getByText('Expirée')).toBeDefined()
		expect(screen.getByText('Clé du fournisseur de tarifs.')).toBeDefined()
		expect(screen.getByText('Statut')).toBeDefined()
		expect(screen.getByText('Utilisation')).toBeDefined()
	})

	it('warns about the nodes that will fail before deleting a used credential', async () => {
		patchCredentialMeta('cred-used', { usedByWorkflows: 1 })
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ id: 'cred-used' })],
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))

		expect(
			await screen.findByText(
				/Utilisée par 1 workflow : leurs nœuds échoueront/,
			),
		).toBeDefined()
	})

	it('keeps the description in the dialog as client-side metadata', async () => {
		patchCredentialMeta('cred-desc', { description: 'Pour les relances.' })
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ id: 'cred-desc' })],
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)

		expect(
			(await screen.findByLabelText(/Description/)) as HTMLTextAreaElement,
		).toHaveProperty('value', 'Pour les relances.')
	})
})

describe('AutomationCredentialsFeature — order of the table', () => {
	it('lists the most recently created identification first', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					credential({
						id: 'old',
						name: 'Ancienne',
						created_at: '2026-01-01T00:00:00Z',
					}),
					credential({
						id: 'new',
						name: 'Récente',
						created_at: '2026-08-01T00:00:00Z',
					}),
					credential({
						id: 'mid',
						name: 'Intermédiaire',
						created_at: '2026-04-01T00:00:00Z',
					}),
				],
			}))
		})

		await screen.findByText('Récente')
		const names = screen
			.getAllByRole('row')
			.slice(1)
			.map((row) => within(row).getAllByRole('button')[0]?.textContent)

		expect(names).toEqual(['Récente', 'Intermédiaire', 'Ancienne'])
	})
})

describe('AutomationCredentialsFeature — type labels', () => {
	it('names the type in French in the table', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					credential({ id: 'a', name: 'Un', kind: 'bearer_token' }),
					credential({ id: 'b', name: 'Deux', kind: 'http_header' }),
					credential({ id: 'c', name: 'Trois', kind: 'odoo_api' }),
				],
			}))
		})

		await screen.findByText('Un')
		expect(screen.getByText('Bearer')).toBeDefined()
		expect(screen.getByText('Clé d’API dans un en-tête')).toBeDefined()
		expect(screen.getByText('Clé d’API Odoo')).toBeDefined()
	})

	it('never shows the technical code while the catalogue is still loading', async () => {
		renderFeature((api) => {
			api.mockGet(CONNECTORS_PATH, () => new Promise(() => {}))
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ kind: 'bearer_token' })],
			}))
		})

		await screen.findByText('Signature sortante')

		expect(screen.getByText('Bearer')).toBeDefined()
		expect(screen.queryByText('bearer_token')).toBeNull()
	})

	it('shows a loading state, not the type picker, when editing before the catalogue is there', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CONNECTORS_PATH, () => new Promise(() => {}))
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByText('Chargement…')).toBeDefined()
		expect(within(dialog).getByText('Signature sortante')).toBeDefined()
		expect(within(dialog).queryByText(/Choisissez comment Mestier/)).toBeNull()
	})
})

describe('AutomationCredentialsFeature — dialog wording', () => {
	it('says where the type can be used, the webhook trigger included', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))

		expect(
			screen.getByText(
				'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
			),
		).toBeDefined()
	})

	it('offers the French categories and types, and finds a type by what it does', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)

		expect(screen.getByText('Requêtes HTTP')).toBeDefined()
		expect(
			screen.getByRole('button', { name: /Clé d’API dans un en-tête/ }),
		).toBeDefined()
		expect(screen.getByRole('button', { name: /Clé d’API Odoo/ })).toBeDefined()

		await user.type(
			screen.getByRole('searchbox', {
				name: 'Rechercher un type d’identification',
			}),
			'http_header',
		)

		expect(screen.getByText(/Aucun type ne correspond/)).toBeDefined()
	})

	it('prefills the header name with its default and labels the fields in French', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(
			screen.getByRole('button', { name: /Clé d’API dans un en-tête/ }),
		)

		expect(
			(screen.getByLabelText('Nom de l’en-tête') as HTMLInputElement).value,
		).toBe('X-API-Key')
		expect(screen.getByLabelText('Valeur')).toBeDefined()
		expect(screen.getByLabelText('Nom')).toHaveProperty(
			'placeholder',
			'Ex. Clé d’API dans un en-tête – fournisseur',
		)
	})

	it('sends the prefilled default along with what was typed', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: credential({ origin: 'supplied' }),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(
			screen.getByRole('button', { name: /Clé d’API dans un en-tête/ }),
		)
		await user.type(screen.getByLabelText('Nom'), 'API tarifs')
		await user.type(screen.getByLabelText('Valeur'), 'sk-123')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		await waitFor(() => {
			expect(
				api.calls.some(
					(c) => c.method === 'post' && c.path === CREDENTIALS_PATH,
				),
			).toBe(true)
		})
		const call = api.calls.find(
			(c) => c.method === 'post' && c.path === CREDENTIALS_PATH,
		)
		expect((call?.params as { body: unknown }).body).toEqual({
			kind: 'http_header',
			name: 'API tarifs',
			origin: 'supplied',
			data: { header_name: 'X-API-Key', header_value: 'sk-123' },
		})
	})

	it('puts the Odoo placeholders in the form', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /Clé d’API Odoo/ }))

		expect(screen.getByLabelText('URL de l’instance')).toHaveProperty(
			'placeholder',
			'https://exemple.odoo.com',
		)
		expect(screen.getByLabelText('Clé d’API')).toBeDefined()
	})

	it('words the edit subtitle like the prototype', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)

		expect(
			await screen.findByText(
				'Modifiez son nom, sa description ou ses paramètres de connexion.',
			),
		).toBeDefined()
		expect(screen.queryByText(/Laissez les champs vides/)).toBeNull()
	})
})

describe('AutomationCredentialsFeature — errors in French', () => {
	it('says the name is taken without the prefix or the constraint name', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, () => {
				throw new Error('Conflict: uq_automation_credential_name')
			})
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))
		await user.type(screen.getByLabelText('Nom'), 'Doublon')
		await user.type(screen.getByLabelText('Jeton'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(
			await screen.findByText(
				'Une identification de ce type porte déjà ce nom.',
			),
		).toBeDefined()
		expect(screen.queryByText(/Conflict/)).toBeNull()
		expect(screen.queryByText(/uq_automation/)).toBeNull()
	})

	it('names the missing field the server complains about by its French label', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, () => {
				throw new Error('Conflict: missing field `token`')
			})
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.click(screen.getByRole('button', { name: /^Bearer/ }))
		await user.type(screen.getByLabelText('Nom'), 'X')
		await user.type(screen.getByLabelText('Jeton'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(
			await screen.findByText('Le champ « Jeton » est obligatoire.'),
		).toBeDefined()
	})

	it('explains a failed list load in French', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => {
				throw new Error('HTTP 500: Internal Server Error')
			})
		})

		expect(
			await screen.findByText(
				'Une erreur est survenue. Réessayez dans un instant.',
			),
		).toBeDefined()
	})
})

describe('AutomationCredentialsFeature — deleting a used credential', () => {
	const refused = () => {
		throw new Error(
			'Conflict: credential is still referenced by workflow(s): Relance devis',
		)
	}

	it('keeps the dialog open, the description and the confirmation when the server refuses', async () => {
		patchCredentialMeta('cred-keep', { description: 'Pour les relances.' })
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ id: 'cred-keep' })],
			}))
			api.mockMutation('delete', CREDENTIAL_PATH, refused)
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		expect(
			await screen.findByText(
				'Impossible de la supprimer : elle est encore utilisée par Relance devis.',
			),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		).toBeDefined()
		expect(credentialMetaOf('cred-keep').description).toBe('Pour les relances.')
		expect(screen.queryByText(/Conflict/)).toBeNull()
	})

	it('does not repeat the refusal in the banner of the list', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('delete', CREDENTIAL_PATH, refused)
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		await screen.findByText(/Impossible de la supprimer/)
		expect(screen.getAllByText(/Impossible de la supprimer/)).toHaveLength(1)
	})

	it('forgets the description and closes the dialog once the server accepts', async () => {
		patchCredentialMeta('cred-gone', { description: 'À oublier.' })
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ id: 'cred-gone' })],
			}))
			api.mockMutation('delete', CREDENTIAL_PATH, () => ({}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		expect(credentialMetaOf('cred-gone').description).toBe('')
	})

	it('does not show a previous refusal when the dialog is opened again', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('delete', CREDENTIAL_PATH, refused)
		})

		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)
		await screen.findByText(/Impossible de la supprimer/)

		await user.keyboard('{Escape}')
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		await user.click(
			await screen.findByRole('button', { name: 'Signature sortante' }),
		)
		await screen.findByRole('dialog')

		expect(screen.queryByText(/Impossible de la supprimer/)).toBeNull()
	})
})

describe('AutomationCredentialsFeature — catalogue not there yet', () => {
	it('shows a loading state, not an empty type picker, when creating before the catalogue is there', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CONNECTORS_PATH, () => new Promise(() => {}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByText('Chargement…')).toBeDefined()
		expect(within(dialog).queryByText(/Aucun type ne correspond/)).toBeNull()
	})
})
