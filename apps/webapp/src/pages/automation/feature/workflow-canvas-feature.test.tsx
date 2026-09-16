import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkflowCanvasFeature } from '#/pages/automation/feature/workflow-canvas-feature'
import { computeFallbackLayout } from '#/pages/automation/lib/layout'
import {
	dragNodeBy,
	installFlowTestEnvironment,
} from '#/pages/automation/test/flow-test-env'

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

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const WORKFLOW_TRIGGER_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/trigger'
const WORKFLOW_VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'

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

const SIMPLE_KIND = 'test.simple'

function connector(
	id: string,
	kind: string = SIMPLE_KIND,
): Schemas.PlacedConnectorDto {
	return { id, kind, version: 1, config: {} }
}

function descriptor(
	kind: string,
	label: string,
	auth: Schemas.AuthRequirementResponse = 'None',
): Schemas.ConnectorDescriptorResponse {
	return {
		auth,
		branches: [],
		family: 'test',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

function credential(
	id: string,
	kind: string,
	name: string,
): Schemas.CredentialResponse {
	return {
		id,
		kind,
		name,
		origin: 'supplied',
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
}

function workflowDetail(
	overrides: Partial<{
		graph: Schemas.GraphDto
		layout: Schemas.WorkflowLayoutDto | null
	}> = {},
) {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Créer une facture Odoo',
		description: null,
		enabled: true,
		layout: overrides.layout ?? null,
		current_version: overrides.graph
			? {
					id: 'version-1',
					workflow_id: 'workflow-1',
					version: 1,
					graph: overrides.graph,
					created_at: '2026-08-01T00:00:00Z',
					created_by: null,
				}
			: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
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
						if (!handler)
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
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

function renderFeature(configure: (api: FakeApi) => void) {
	const api = installFakeTanstackApi()
	api.mockGet(CONNECTORS_PATH, () => ({
		data: {
			auth_schemes: [],
			connectors: [descriptor(SIMPLE_KIND, 'Étape simple')],
		},
		pagination: null,
	}))
	api.mockGet(WORKFLOW_TRIGGER_PATH, () => ({
		data: { event_names: ['quote.accepted'] },
		pagination: null,
	}))
	api.mockGet(WORKFLOW_PATH, () => ({
		data: workflowDetail(),
		pagination: null,
	}))
	api.mockGet(CREDENTIALS_PATH, () => ({
		data: [],
		pagination: null,
	}))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

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
			<WorkflowCanvasFeature workflowId="workflow-1" />
		</Providers>,
	)

	return api
}

beforeEach(() => {
	installFlowTestEnvironment()
})

async function clickSave() {
	const button = await screen.findByRole('button', { name: /Enregistrer/ })
	button.click()
}

describe('WorkflowCanvasFeature — loading the workflow', () => {
	it('renders the graph once the workflow, catalogue and trigger have loaded', async () => {
		renderFeature((api) => {
			api.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [connector('c1')], edges: [] },
					layout: { c1: { x: 40, y: 20 } },
				}),
				pagination: null,
			}))
		})

		expect(await screen.findByText('Étape simple')).toBeDefined()
	})
})

describe('WorkflowCanvasFeature — saving', () => {
	it('saves a fully-positioned graph back unchanged', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
		}
		const layout = { c1: { x: 0, y: 0 }, c2: { x: 280, y: 0 } }

		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
				data: {
					id: 'version-2',
					workflow_id: 'workflow-1',
					version: 2,
					graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
					created_at: '2026-08-02T00:00:00Z',
					created_by: null,
				},
			}))
		})

		await clickSave()

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
			)
			expect(call).toBeDefined()
		})

		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		const body = (call?.params as { body: Schemas.SaveWorkflowVersionRequest })
			.body
		expect(body.graph).toEqual(graph)
		expect(body.layout).toEqual(layout)
	})

	it('places a connector with no stored position before saving it, instead of stacking it at the origin', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
		}
		const layout = { c1: { x: 0, y: 0 } }

		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
				data: {
					id: 'version-2',
					workflow_id: 'workflow-1',
					version: 2,
					graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
					created_at: '2026-08-02T00:00:00Z',
					created_by: null,
				},
			}))
		})

		await clickSave()

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
			)
			expect(call).toBeDefined()
		})

		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		const body = (call?.params as { body: Schemas.SaveWorkflowVersionRequest })
			.body
		const expectedFallback = computeFallbackLayout(graph, new Set(['c1']))
		expect(body.layout?.c2).toEqual(expectedFallback.get('c2'))
		expect(body.layout?.c2).not.toEqual({ x: 0, y: 0 })
	})

	it('shows the dirty indicator after a change and clears it once saved', async () => {
		const graph: Schemas.GraphDto = { connectors: [connector('c1')], edges: [] }
		const layout = { c1: { x: 0, y: 0 } }

		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
				data: {
					id: 'version-2',
					workflow_id: 'workflow-1',
					version: 2,
					graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
					created_at: '2026-08-02T00:00:00Z',
					created_by: null,
				},
			}))
		})

		expect(screen.queryByText(/non enregistr/i)).toBeNull()

		const node = await screen.findByTestId('rf__node-c1')
		dragNodeBy(node, 40, 30)

		await waitFor(() => {
			expect(screen.getByText(/non enregistr/i)).toBeDefined()
		})

		await clickSave()

		await waitFor(() => {
			expect(screen.queryByText(/non enregistr/i)).toBeNull()
		})
	})
})

describe('WorkflowCanvasFeature — a 422 from the backend', () => {
	it('badges the named connector and banners the graph-shaped errors, without a toast', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
		}
		const layout = { c1: { x: 0, y: 0 }, c2: { x: 280, y: 0 } }

		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('Invalid graph'), {
					status: 422,
					data: {
						code: 'graph_invalid',
						message: 'Invalid graph',
						status: 422,
						details: {
							errors: [
								{
									connector_id: 'c1',
									field: null,
									message: 'Identifiant de credential manquant',
								},
								{
									connector_id: null,
									field: null,
									message: 'Le graphe contient un cycle',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()

		const node = await screen.findByTestId('rf__node-c1')
		expect(await within(node).findByLabelText(/Erreur/)).toBeDefined()
		expect(await screen.findByText('Le graphe contient un cycle')).toBeDefined()
	})

	it('lands a field-scoped 422 on the exact field of the exact node, in the config panel', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1'),
				{ id: 'c2', kind: 'test.withfield', version: 1, config: {} },
			],
			edges: [],
		}
		const layout = { c1: { x: 0, y: 0 }, c2: { x: 280, y: 0 } }

		renderFeature((fakeApi) => {
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [],
					connectors: [
						descriptor(SIMPLE_KIND, 'Étape simple'),
						{
							...descriptor('test.withfield', 'Avec un champ'),
							fields: [
								{
									name: 'url',
									label: 'URL',
									kind: 'Text',
									required: true,
									secret: false,
									expression: true,
									visible_when: null,
								},
							],
						},
					],
				},
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('Invalid graph'), {
					status: 422,
					data: {
						code: 'graph_invalid',
						message: 'Invalid graph',
						status: 422,
						details: {
							errors: [
								{
									connector_id: 'c2',
									field: 'url',
									message: 'URL manquante',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()

		const node = await screen.findByTestId('rf__node-c2')
		node.click()

		expect(await screen.findByText('URL manquante')).toBeDefined()
	})

	it('says the save failed when the refusal carries no graph errors', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [],
		}

		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout: { c1: { x: 0, y: 0 } } }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('Service Unavailable'), { status: 503 })
			})
		})

		await clickSave()

		expect(await screen.findByRole('alert')).toBeDefined()
		expect(await screen.findByText(/enregistrement a échoué/i)).toBeDefined()
	})

	it('clears the failure message once a later save succeeds', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [],
		}
		let attempts = 0

		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout: { c1: { x: 0, y: 0 } } }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				attempts += 1
				if (attempts === 1) {
					throw Object.assign(new Error('Service Unavailable'), {
						status: 503,
					})
				}
				return { data: null, pagination: null }
			})
		})

		await clickSave()
		expect(await screen.findByText(/enregistrement a échoué/i)).toBeDefined()

		await clickSave()

		await waitFor(() => {
			expect(screen.queryByText(/enregistrement a échoué/i)).toBeNull()
		})
	})
})

describe('WorkflowCanvasFeature — the credential picker', () => {
	it('offers the credentials fetched for real, filtered by the connector auth', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [
						{ kind: 'bearer_token', label: 'Bearer token', fields: [] },
					],
					connectors: [
						descriptor(SIMPLE_KIND, 'Étape simple', {
							Exactly: 'bearer_token',
						}),
					],
				},
				pagination: null,
			}))
			api.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [connector('c1')], edges: [] },
					layout: { c1: { x: 0, y: 0 } },
				}),
				pagination: null,
			}))
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential('cred-1', 'bearer_token', 'Ma clé')],
				pagination: null,
			}))
		})

		const node = await screen.findByTestId('rf__node-c1')
		node.click()

		await user.click(
			await screen.findByRole('combobox', { name: 'Identification' }),
		)

		expect(await screen.findByRole('option', { name: 'Ma clé' })).toBeDefined()
	})
})

describe('WorkflowCanvasFeature — inline credential creation', () => {
	it('creates the credential through the real mutation and saves the connector pointing at it', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [
						{
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
						},
					],
					connectors: [
						descriptor(SIMPLE_KIND, 'Étape simple', {
							Exactly: 'bearer_token',
						}),
					],
				},
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [connector('c1')], edges: [] },
					layout: { c1: { x: 0, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('post', CREDENTIALS_PATH, (params) => ({
				data: {
					...credential(
						'new-cred',
						'bearer_token',
						(params as { body: Schemas.CreateCredentialRequest }).body.name,
					),
					secret: 'irrelevant',
				},
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
				data: {
					id: 'version-2',
					workflow_id: 'workflow-1',
					version: 2,
					graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
					created_at: '2026-08-02T00:00:00Z',
					created_by: null,
				},
			}))
		})

		const node = await screen.findByTestId('rf__node-c1')
		node.click()

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Token'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'post' && c.path === CREDENTIALS_PATH,
			)
			expect(call).toBeDefined()
		})

		await clickSave()

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
			)
			expect(call).toBeDefined()
		})

		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		const body = (call?.params as { body: Schemas.SaveWorkflowVersionRequest })
			.body
		expect(body.graph.connectors[0]?.credential_id).toBe('new-cred')
	})
})
