import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useRouterState } from '@tanstack/react-router'
import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkflowPageFeature } from '#/pages/automation/feature/workflow-page-feature'
import { computeFallbackLayout } from '#/pages/automation/lib/layout'
import {
	dragNodeBy,
	installFlowTestEnvironment,
} from '#/pages/automation/test/flow-test-env'
import { renderWithRouter } from '#/test/render-with-router'

vi.mock('#/pages/automation/feature/workflow-executions-feature', async () => {
	const { WorkflowTopBarFeature } = await import(
		'#/pages/automation/feature/workflow-top-bar-feature'
	)
	return {
		WorkflowExecutionsFeature: ({ workflowId }: { workflowId: string }) => (
			<div>
				<WorkflowTopBarFeature workflowId={workflowId} view="executions" />
				<p>exécutions de {workflowId}</p>
			</div>
		),
	}
})

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
const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const EXPRESSIONS_EVALUATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/expressions/evaluate'

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

function event(
	name: string,
	payloadExample: unknown = { id: 1 },
): Schemas.EventDescriptorResponse {
	return {
		name,
		label: name,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: payloadExample,
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

const WORKFLOW_URL = '/o/atelier-bois/automatisation/workflows/workflow-1'

function UrlDrivenPage() {
	const location = useRouterState({ select: (state) => state.location })
	const workflowId = location.pathname.split('/').at(-1) ?? ''
	const search = location.search as { vue?: string }

	return (
		<WorkflowPageFeature
			workflowId={workflowId}
			view={search.vue === 'executions' ? 'executions' : 'editor'}
		/>
	)
}

function renderFeature(configure: (api: FakeApi) => void) {
	return renderPage(configure).api
}

function renderPage(
	configure: (api: FakeApi) => void,
	initialPath = WORKFLOW_URL,
) {
	const api = installFakeTanstackApi()
	api.mockGet(CONNECTORS_PATH, () => ({
		data: {
			auth_schemes: [],
			connectors: [descriptor(SIMPLE_KIND, 'Étape simple')],
		},
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
	api.mockGet(EVENTS_PATH, () => ({
		data: [],
		pagination: null,
	}))
	api.mockGet(RUNS_PATH, () => ({
		data: [],
		pagination: null,
	}))
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions: ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'] },
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
						<TooltipProvider>{children}</TooltipProvider>
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	const rendered = renderWithRouter(
		<Providers>
			<UrlDrivenPage />
		</Providers>,
		initialPath,
	)

	return { api, rendered }
}

beforeEach(() => {
	installFlowTestEnvironment()
})

async function closeDetails() {
	fireEvent.click(
		await screen.findByRole('button', { name: 'Fermer les détails du nœud' }),
	)
	await waitFor(() => {
		expect(screen.queryByRole('dialog')).toBeNull()
	})
}

async function clickSave() {
	await waitFor(() => {
		const button = screen.getByRole('button', {
			name: /Enregistrer/,
		}) as HTMLButtonElement
		expect(button.disabled).toBe(false)
	})
	screen.getByRole('button', { name: /Enregistrer/ }).click()
}

describe('WorkflowPageFeature (editor) — loading the workflow', () => {
	it('renders the graph once the workflow and catalogue have loaded', async () => {
		renderFeature((api) => {
			api.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [connector('c1')], edges: [], triggers: [] },
					layout: { c1: { x: 40, y: 20 } },
				}),
				pagination: null,
			}))
		})

		expect(await screen.findByText('Étape simple')).toBeDefined()
	})
})

describe('WorkflowPageFeature (editor) — saving', () => {
	it('saves a fully-positioned graph back unchanged', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
			triggers: [],
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
			triggers: [],
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
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [],
			triggers: [],
		}
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

describe('WorkflowPageFeature (editor) — a 422 from the backend', () => {
	it('badges the named connector and banners the graph-shaped errors, without a toast', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
			triggers: [],
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
		expect(
			await within(node).findByLabelText(
				'À corriger : Identifiant de credential manquant',
			),
		).toBeDefined()
		expect(await screen.findByText('Le graphe contient un cycle')).toBeDefined()
	})

	it('lands a field-scoped 422 on the exact field of the exact node, in the node details', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1'),
				{ id: 'c2', kind: 'test.withfield', version: 1, config: {} },
			],
			edges: [],
			triggers: [],
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
		fireEvent.doubleClick(node)

		expect(await screen.findByText('URL manquante')).toBeDefined()
	})

	it('lands a trigger-scoped 422 on the trigger wired to nothing, not on the graph banner', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
		}
		const layout = { t1: { x: 0, y: 0 } }

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
									connector_id: null,
									trigger_id: 't1',
									field: null,
									message: 'Ce déclencheur ne mène nulle part.',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()

		expect(screen.queryByText('Ce déclencheur ne mène nulle part.')).toBeNull()

		const node = await screen.findByTestId('rf__node-t1')
		expect(
			await within(node).findByLabelText(
				'À corriger : Ce déclencheur ne mène nulle part.',
			),
		).toBeDefined()

		fireEvent.doubleClick(node)
		expect(
			await screen.findByText('Ce déclencheur ne mène nulle part.'),
		).toBeDefined()
	})

	it('says the save failed when the refusal carries no graph errors', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [],
			triggers: [],
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
			triggers: [],
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

describe('WorkflowPageFeature (editor) — the credential picker', () => {
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
					graph: { connectors: [connector('c1')], edges: [], triggers: [] },
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
		fireEvent.doubleClick(node)

		await user.click(
			await screen.findByRole('combobox', { name: 'Identification' }),
		)

		expect(await screen.findByRole('option', { name: 'Ma clé' })).toBeDefined()
	})
})

describe('WorkflowPageFeature (editor) — inline credential creation', () => {
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
					graph: { connectors: [connector('c1')], edges: [], triggers: [] },
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
		fireEvent.doubleClick(node)

		await user.click(
			await screen.findByRole('combobox', { name: 'Identification' }),
		)
		await user.click(
			await screen.findByRole('button', {
				name: /Créer|Nouvelle identification/,
			}),
		)
		await user.type(screen.getByLabelText('Nom'), 'Ma clé')
		await user.type(screen.getByLabelText('Jeton'), 's3cr3t')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'post' && c.path === CREDENTIALS_PATH,
			)
			expect(call).toBeDefined()
		})

		await closeDetails()
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

describe('WorkflowPageFeature (editor) — a trigger saved with the workflow', () => {
	it('edits a real trigger node and saves it through the same PUT as the graph, with no legacy trigger endpoint involved', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
		}
		const layout = { c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 } }

		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(EVENTS_PATH, () => ({
				data: [event('quote.accepted'), event('invoice.paid')],
				pagination: null,
			}))
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

		fireEvent.doubleClick(await screen.findByTestId('rf__node-t1'))

		fireEvent.click(
			await screen.findByRole('button', { name: 'Ajouter un événement' }),
		)
		fireEvent.click(
			await screen.findByRole('button', { name: /invoice\.paid/ }),
		)
		fireEvent.click(
			screen.getByRole('button', { name: 'Ajouter cet événement' }),
		)

		await closeDetails()
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
		expect(body.graph.triggers).toEqual([
			{ id: 't1', kind: { Events: ['quote.accepted', 'invoice.paid'] } },
		])
	})

	it('adds a second trigger from the pane menu and saves both alongside the connectors', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [{ id: 't1', kind: 'Manual' }],
		}
		const layout = { c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 } }

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

		await screen.findByTestId('rf__node-t1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 400, clientY: 400 })
		fireEvent.click(await screen.findByText('Ajouter un nœud'))
		fireEvent.click(await screen.findByText('Ajouter un déclencheur'))
		fireEvent.click(
			within(
				await screen.findByRole('complementary', { name: 'Créateur de nœuds' }),
			).getByRole('button', { name: /Clic/ }),
		)
		await screen.findByTestId('rf__node-t2')

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
		expect(body.graph.triggers).toEqual([
			{ id: 't1', kind: 'Manual' },
			{ id: 't2', kind: 'Manual' },
		])
	})
})

describe('WorkflowPageFeature (editor) — the last run', () => {
	it('fills the input column with the most recent run for this workflow, and none other', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1')],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [{ id: 't1', kind: 'Manual' }],
		}

		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph,
					layout: { t1: { x: -220, y: 0 }, c1: { x: 0, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockGet(RUNS_PATH, () => ({
				data: [
					{
						id: 'run-old',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-01T00:00:00Z',
					},
					{
						id: 'run-new',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-05T00:00:00Z',
					},
					{
						id: 'run-other-workflow',
						organization_id: 'org-1',
						workflow_id: 'workflow-2',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-09T00:00:00Z',
					},
				],
				pagination: null,
			}))
			fakeApi.mockGet(RUN_PATH, (params) => {
				const runId = (params as { path: { run_id: string } }).path.run_id
				expect(runId).toBe('run-new')
				return {
					data: {
						id: 'run-new',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-05T00:00:00Z',
						trigger_payload: { quote_id: 'q-real' },
						steps: [
							{
								id: 'step-1',
								connector_id: 'c1',
								iteration_path: '',
								attempts: 1,
								status: 'succeeded',
								created_at: '2026-08-05T00:00:01Z',
								output: { id: 42 },
							},
						],
					},
					pagination: null,
				}
			})
		})

		fireEvent.doubleClick(await screen.findByTestId('rf__node-c1'))

		fireEvent.click(await screen.findByRole('radio', { name: 'Exécution' }))

		expect(await screen.findByText('q-real')).toBeDefined()
	})
})

describe('WorkflowPageFeature (editor) — the live preview', () => {
	it('resolves an expression through the real evaluate endpoint', async () => {
		const withField = {
			...descriptor(SIMPLE_KIND, 'Étape simple'),
			fields: [
				{
					name: 'url',
					label: 'URL',
					kind: 'Text' as const,
					required: true,
					secret: false,
					expression: true,
					visible_when: null,
				},
			],
		}

		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: { auth_schemes: [], connectors: [withField] },
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: {
						connectors: [
							{ ...connector('c1'), config: { url: '{{ trigger.id }}' } },
						],
						edges: [],
						triggers: [],
					},
					layout: { c1: { x: 0, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('post', EXPRESSIONS_EVALUATE_PATH, () => ({
				data: { value: 'resolved!' },
				pagination: null,
			}))
		})

		fireEvent.doubleClick(await screen.findByTestId('rf__node-c1'))

		fireEvent.focus(await screen.findByLabelText('URL'))

		await waitFor(
			() => {
				expect(screen.getByText('Aperçu : resolved!')).toBeDefined()
			},
			{ timeout: 2000 },
		)

		const call = api.calls.find(
			(c) => c.method === 'post' && c.path === EXPRESSIONS_EVALUATE_PATH,
		)
		expect(call).toBeDefined()
	})
})

interface ServerState {
	name: string
	enabled: boolean
}

function mockWorkflowServer(
	api: FakeApi,
	server: ServerState,
	graph?: Schemas.GraphDto,
) {
	api.mockGet(WORKFLOW_PATH, () => ({
		data: {
			...workflowDetail(graph ? { graph } : {}),
			name: server.name,
			enabled: server.enabled,
		},
		pagination: null,
	}))
	api.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
		data: {
			id: 'version-2',
			workflow_id: 'workflow-1',
			version: 2,
			graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
			created_at: '2026-08-02T00:00:00Z',
			created_by: null,
		},
	}))
	api.mockMutation('patch', WORKFLOW_PATH, (params) => {
		const body = (params as { body: Partial<ServerState> }).body
		server.name = body.name ?? server.name
		server.enabled = body.enabled ?? server.enabled
		return { data: { id: 'workflow-1', ...server }, pagination: null }
	})
}

function mutationCalls(api: FakeApi) {
	return api.calls.filter((call) => call.method !== 'get')
}

function workflowReads(api: FakeApi, workflowId: string) {
	return api.calls.filter(
		(call) =>
			call.method === 'get' &&
			call.path === WORKFLOW_PATH &&
			(call.params as { path: { workflow_id: string } }).path.workflow_id ===
				workflowId,
	)
}

function nameField(): HTMLInputElement {
	return screen.getByLabelText('Nom du workflow') as HTMLInputElement
}

function activeSwitch(): HTMLElement {
	return screen.getByRole('switch', { name: 'Actif' })
}

describe('WorkflowPageFeature (editor) — the name and the switch', () => {
	it('keeps a rename in the draft until « Enregistrer », then sends it alone and refreshes the workflow', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		const api = renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		const input = await waitFor(() => {
			const field = nameField()
			expect(field.value).toBe('Relance des devis')
			return field
		})
		const readsBefore = workflowReads(api, 'workflow-1').length
		await user.clear(input)
		await user.type(input, 'Relance 2')

		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		expect(mutationCalls(api)).toEqual([])

		await clickSave()

		await waitFor(() => {
			expect(screen.getByText('Enregistré')).toBeDefined()
		})
		expect(mutationCalls(api).map((call) => call.method)).toEqual(['patch'])
		expect(mutationCalls(api)[0].params).toMatchObject({
			path: { workflow_id: 'workflow-1' },
			body: { name: 'Relance 2' },
		})
		expect(nameField().value).toBe('Relance 2')
		expect(workflowReads(api, 'workflow-1').length).toBeGreaterThan(readsBefore)
	})

	it('takes the switch into the draft too, and sends only the field that changed', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		const api = renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		await waitFor(() => {
			expect(activeSwitch().getAttribute('aria-checked')).toBe('true')
		})
		await user.click(activeSwitch())

		expect(activeSwitch().getAttribute('aria-checked')).toBe('false')
		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		expect(mutationCalls(api)).toEqual([])

		await clickSave()

		await waitFor(() => {
			expect(screen.getByText('Enregistré')).toBeDefined()
		})
		expect(mutationCalls(api).map((call) => call.method)).toEqual(['patch'])
		expect(mutationCalls(api)[0].params).toMatchObject({
			body: { enabled: false },
		})
		expect(
			(mutationCalls(api)[0].params as { body: object }).body,
		).not.toHaveProperty('name')
		expect(server.enabled).toBe(false)
		expect(activeSwitch().getAttribute('aria-checked')).toBe('false')
	})

	it('saves the graph first, then the name, when both changed', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		const api = renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		const input = await waitFor(() => {
			const field = nameField()
			expect(field.value).toBe('Relance des devis')
			return field
		})
		await user.click(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		)
		await user.click(await screen.findByRole('button', { name: /Clic/ }))
		await screen.findByTestId('rf__node-t1')
		await user.clear(input)
		await user.type(input, 'Autre nom')

		await clickSave()

		await waitFor(() => {
			expect(mutationCalls(api).map((call) => call.method)).toEqual([
				'put',
				'patch',
			])
		})
	})

	it('says so when the name or the switch could not be saved, and keeps what was typed', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		renderFeature((fakeApi) => {
			mockWorkflowServer(fakeApi, server)
			fakeApi.mockMutation('patch', WORKFLOW_PATH, () => {
				throw Object.assign(new Error('Service Unavailable'), { status: 503 })
			})
		})

		const input = await waitFor(() => {
			const field = nameField()
			expect(field.value).toBe('Relance des devis')
			return field
		})
		await user.clear(input)
		await user.type(input, 'Relance 2')
		await clickSave()

		expect((await screen.findByRole('alert')).textContent).toContain(
			'L’enregistrement du nom ou de l’état du workflow a échoué : Service Unavailable',
		)
		expect(nameField().value).toBe('Relance 2')
		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
	})

	it('puts the previous name back when the field is left empty', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		const input = await waitFor(() => {
			const field = nameField()
			expect(field.value).toBe('Relance des devis')
			return field
		})
		await user.clear(input)
		await user.tab()

		expect(nameField().value).toBe('Relance des devis')
		expect(screen.getByText('Enregistré')).toBeDefined()
	})
})

describe('WorkflowPageFeature (editor) — switching to the executions and back', () => {
	it('keeps the unsaved graph and the unsaved state across the round trip', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		const api = renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		await user.click(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		)
		await user.click(await screen.findByRole('button', { name: /Clic/ }))
		await screen.findByTestId('rf__node-t1')
		const input = nameField()
		await user.clear(input)
		await user.type(input, 'Relance 2')

		await user.click(screen.getByRole('tab', { name: 'Exécutions' }))

		expect(await screen.findByText('exécutions de workflow-1')).toBeDefined()
		expect(screen.queryByTestId('rf__node-t1')).toBeNull()
		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		expect(nameField().value).toBe('Relance 2')

		await user.click(screen.getByRole('tab', { name: 'Éditeur' }))

		expect(await screen.findByTestId('rf__node-t1')).toBeDefined()
		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		expect(nameField().value).toBe('Relance 2')
		expect(mutationCalls(api)).toEqual([])
	})

	it('can save from the executions view', async () => {
		const user = userEvent.setup()
		const server = { name: 'Relance des devis', enabled: true }
		const api = renderFeature((fakeApi) => mockWorkflowServer(fakeApi, server))

		await user.click(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		)
		await user.click(await screen.findByRole('button', { name: /Clic/ }))
		await screen.findByTestId('rf__node-t1')
		await user.click(screen.getByRole('tab', { name: 'Exécutions' }))
		await screen.findByText('exécutions de workflow-1')

		await clickSave()

		await waitFor(() => {
			expect(mutationCalls(api).map((call) => call.method)).toEqual(['put'])
		})
		const body = (
			mutationCalls(api)[0].params as {
				body: Schemas.SaveWorkflowVersionRequest
			}
		).body
		expect(body.graph.triggers).toEqual([{ id: 't1', kind: 'Manual' }])
	})
})

describe('WorkflowPageFeature (editor) — errors that go away once fixed', () => {
	it('lifts the badge of a node as soon as its settings change, and leaves the others alone', async () => {
		const user = userEvent.setup()
		const withField = {
			...descriptor('test.withfield', 'Avec un champ'),
			fields: [
				{
					name: 'url',
					label: 'URL',
					kind: 'Text' as const,
					required: true,
					secret: false,
					expression: false,
					visible_when: null,
				},
			],
		}
		const graph: Schemas.GraphDto = {
			connectors: [
				{ id: 'c1', kind: 'test.withfield', version: 1, config: {} },
				{ id: 'c2', kind: 'test.withfield', version: 1, config: {} },
			],
			edges: [],
			triggers: [],
		}
		renderFeature((fakeApi) => {
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: { auth_schemes: [], connectors: [withField] },
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph,
					layout: { c1: { x: 0, y: 0 }, c2: { x: 280, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('Invalid graph'), {
					status: 422,
					data: {
						details: {
							errors: [
								{ connector_id: 'c1', field: 'url', message: 'URL manquante' },
								{ connector_id: 'c2', field: 'url', message: 'URL manquante' },
							],
						},
					},
				})
			})
		})

		await clickSave()
		const first = await screen.findByTestId('rf__node-c1')
		await within(first).findByLabelText('À corriger : URL manquante')

		fireEvent.doubleClick(first)
		await user.type(await screen.findByLabelText('URL'), 'https://a.example')
		await closeDetails()

		await waitFor(() => {
			expect(
				within(screen.getByTestId('rf__node-c1')).queryByLabelText(
					/À corriger/,
				),
			).toBeNull()
		})
		expect(
			within(screen.getByTestId('rf__node-c2')).getByLabelText(
				'À corriger : URL manquante',
			),
		).toBeDefined()
	})
})

describe('WorkflowPageFeature (editor) — a workflow that cannot be opened', () => {
	it('shows the not-found screen, without the editor bar, for an unknown id', async () => {
		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => {
				throw Object.assign(new Error('Not found'), { status: 404 })
			})
		})

		expect(
			await screen.findByRole('heading', { name: 'Workflow introuvable' }),
		).toBeDefined()
		expect(screen.getByText(/« workflow-1 »/)).toBeDefined()
		expect(
			screen
				.getByRole('link', { name: 'Retour aux workflows' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/workflows')
		expect(screen.queryByRole('tab', { name: 'Exécutions' })).toBeNull()
	})

	it('shows the error and a way back for any other failure', async () => {
		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => {
				throw Object.assign(new Error('Service Unavailable'), { status: 503 })
			})
		})

		expect(
			await screen.findByText('Impossible de charger le workflow'),
		).toBeDefined()
		expect(screen.getByText('Service Unavailable')).toBeDefined()
		expect(
			screen
				.getByRole('link', { name: 'Retour aux workflows' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/workflows')
		expect(
			screen.queryByRole('heading', { name: 'Workflow introuvable' }),
		).toBeNull()
	})
})

describe('WorkflowPageFeature (editor) — the new workflow draft', () => {
	const DRAFT_URL = '/o/atelier-bois/automatisation/workflows/nouveau'

	function mockCreation(api: FakeApi) {
		api.mockMutation('post', WORKFLOWS_PATH, (params) => ({
			data: {
				id: 'workflow-9',
				organization_id: 'org-1',
				name: (params as { body: { name: string } }).body.name,
				description: null,
				enabled: true,
				current_version_id: null,
				created_at: '2026-08-02T00:00:00Z',
				updated_at: '2026-08-02T00:00:00Z',
			},
			pagination: null,
		}))
		api.mockMutation('put', WORKFLOW_VERSIONS_PATH, (params) => ({
			data: {
				id: 'version-1',
				workflow_id: 'workflow-9',
				version: 1,
				graph: (params as { body: { graph: Schemas.GraphDto } }).body.graph,
				created_at: '2026-08-02T00:00:00Z',
				created_by: null,
			},
		}))
		api.mockMutation('patch', WORKFLOW_PATH, () => ({
			data: { id: 'workflow-9' },
			pagination: null,
		}))
	}

	async function addClickTrigger(user: ReturnType<typeof userEvent.setup>) {
		await user.click(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		)
		await user.click(await screen.findByRole('button', { name: /Clic/ }))
		return screen.findByTestId('rf__node-t1')
	}

	it('opens an empty editor named « Mon workflow », inactive, without asking the server for it', async () => {
		const api = renderPage(mockCreation, DRAFT_URL).api

		expect(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		).toBeDefined()
		expect(nameField().value).toBe('Mon workflow')
		expect(activeSwitch().getAttribute('aria-checked')).toBe('false')
		expect(workflowReads(api, 'nouveau')).toEqual([])
	})

	it('creates the workflow, saves its version, keeps it inactive, then swaps the address without reloading the editor', async () => {
		const user = userEvent.setup()
		const rendered = renderPage(mockCreation, DRAFT_URL)
		const node = await addClickTrigger(user)

		await clickSave()

		const { router } = await rendered.rendered
		await waitFor(() => {
			expect(router.state.location.pathname).toBe(
				'/o/atelier-bois/automatisation/workflows/workflow-9',
			)
		})
		expect(mutationCalls(rendered.api).map((call) => `${call.method}`)).toEqual(
			['post', 'put', 'patch'],
		)
		expect(mutationCalls(rendered.api)[0].params).toMatchObject({
			body: { name: 'Mon workflow' },
		})
		expect(mutationCalls(rendered.api)[1].params).toMatchObject({
			path: { workflow_id: 'workflow-9' },
			body: { graph: { triggers: [{ id: 't1', kind: 'Manual' }] } },
		})
		expect(mutationCalls(rendered.api)[2].params).toMatchObject({
			path: { workflow_id: 'workflow-9' },
			body: { enabled: false },
		})
		expect(screen.getByTestId('rf__node-t1')).toBe(node)
		expect(screen.getByText('Enregistré')).toBeDefined()
		expect(workflowReads(rendered.api, 'nouveau')).toEqual([])
		expect(screen.queryByText('Chargement du workflow…')).toBeNull()
	})

	it('has no need to deactivate a workflow the user switched on', async () => {
		const user = userEvent.setup()
		const rendered = renderPage(mockCreation, DRAFT_URL)
		await addClickTrigger(user)
		await user.click(activeSwitch())

		await clickSave()

		const { router } = await rendered.rendered
		await waitFor(() => {
			expect(router.state.location.pathname).toContain('workflow-9')
		})
		expect(mutationCalls(rendered.api).map((call) => `${call.method}`)).toEqual(
			['post', 'put'],
		)
	})

	it('names the workflow after what was typed in the bar', async () => {
		const user = userEvent.setup()
		const rendered = renderPage(mockCreation, DRAFT_URL)
		await addClickTrigger(user)
		await user.clear(nameField())
		await user.type(nameField(), 'Relance 3')

		await clickSave()

		await waitFor(() => {
			expect(mutationCalls(rendered.api)[0]?.params).toMatchObject({
				body: { name: 'Relance 3' },
			})
		})
	})

	it('creates the workflow once even when its graph is refused, and saves the version again on the next try', async () => {
		const user = userEvent.setup()
		let attempts = 0
		const rendered = renderPage((fakeApi) => {
			mockCreation(fakeApi)
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				attempts += 1
				if (attempts === 1) {
					throw Object.assign(new Error('Invalid graph'), {
						status: 422,
						data: {
							details: {
								errors: [
									{
										connector_id: null,
										trigger_id: 't1',
										field: null,
										message: 'Ce déclencheur ne mène nulle part.',
									},
								],
							},
						},
					})
				}
				return { data: null, pagination: null }
			})
		}, DRAFT_URL)
		const node = await addClickTrigger(user)

		await clickSave()
		await within(node).findByLabelText(
			'À corriger : Ce déclencheur ne mène nulle part.',
		)
		const { router } = await rendered.rendered
		expect(router.state.location.pathname).toBe(
			'/o/atelier-bois/automatisation/workflows/nouveau',
		)

		await clickSave()

		await waitFor(() => {
			expect(router.state.location.pathname).toContain('workflow-9')
		})
		expect(
			mutationCalls(rendered.api).filter((call) => call.method === 'post'),
		).toHaveLength(1)
	})

	it('starts from a blank draft again when « nouveau » is opened after a creation', async () => {
		const user = userEvent.setup()
		const rendered = renderPage(mockCreation, DRAFT_URL)
		await addClickTrigger(user)
		await clickSave()
		const { router } = await rendered.rendered
		await waitFor(() => {
			expect(router.state.location.pathname).toContain('workflow-9')
		})

		router.history.push(DRAFT_URL)

		expect(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		).toBeDefined()
		expect(nameField().value).toBe('Mon workflow')
		expect(screen.queryByTestId('rf__node-t1')).toBeNull()
	})

	it('cannot be tested before it exists', async () => {
		const user = userEvent.setup()
		renderPage(mockCreation, DRAFT_URL)
		await addClickTrigger(user)

		expect(
			(
				screen.getByRole('button', {
					name: 'Tester le workflow',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('says there is nothing to list in the executions of a draft', async () => {
		const user = userEvent.setup()
		renderPage(mockCreation, DRAFT_URL)
		await screen.findByRole('button', { name: 'Ajouter la première étape' })

		await user.click(screen.getByRole('tab', { name: 'Exécutions' }))

		expect(
			await screen.findByText(
				'Aucune exécution : ce workflow n’est pas encore enregistré.',
			),
		).toBeDefined()
	})
})
