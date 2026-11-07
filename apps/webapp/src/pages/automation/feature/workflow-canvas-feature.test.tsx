import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useRouterState } from '@tanstack/react-router'
import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
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
	BEARER_TOKEN,
	credential as fixture,
	HTTP_HEADER,
} from '#/pages/automation/test/credential-fixtures'
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
const WORKFLOW_DRY_RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/dry-run'
const WORKFLOW_VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const CREDENTIAL_KINDS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credential-kinds'
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
	mcp_enabled: false,
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
		execution: 'per_item',
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
	return fixture({ id, kind, name })
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
	api.mockGet(CREDENTIAL_KINDS_PATH, () => ({
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

	rendered.catch(() => undefined)

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

	it('says in the node details, in French, that the name is already taken', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				{ ...connector('c1'), name: 'Facturer' },
				{ ...connector('c2'), name: 'facturer' },
			],
			edges: [],
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
						code: 'E_GRAPH_INVALID',
						message: 'Invalid graph',
						status: 422,
						details: {
							errors: [
								{
									connector_id: 'c2',
									field: 'name',
									message:
										'connector `c2` reuses the name `facturer`, already given to another node',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()

		fireEvent.doubleClick(await screen.findByTestId('rf__node-c2'))
		const dialog = await screen.findByRole('dialog')

		expect((await within(dialog).findByRole('alert')).textContent).toBe(
			'Un autre nœud porte déjà ce nom.',
		)
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
			api.mockGet(CREDENTIAL_KINDS_PATH, () => ({
				data: [BEARER_TOKEN],
				pagination: null,
			}))
			api.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [],
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
			fakeApi.mockGet(CREDENTIAL_KINDS_PATH, () => ({
				data: [BEARER_TOKEN],
				pagination: null,
			}))
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [],
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
			{ id: 't2', kind: 'Manual', name: 'Clic 2' },
		])
	})
})

describe('WorkflowPageFeature (editor) — the node fields', () => {
	it('shows the fields saved with the graph and sends a new name back with the next version', async () => {
		const user = userEvent.setup()
		const saved: Schemas.PlacedConnectorDto = {
			...connector('c1'),
			name: 'Facturer',
			description: 'Crée la facture',
			disabled: true,
		}
		const api = renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [saved], edges: [], triggers: [] },
					layout: { c1: { x: 0, y: 0 } },
				}),
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

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).getByText('Facturer')).toBeDefined()
		expect(within(node).getByText('Crée la facture')).toBeDefined()
		expect(within(node).getByText('Désactivé')).toBeDefined()

		fireEvent.contextMenu(node, { clientX: 200, clientY: 150 })
		await user.click(await screen.findByRole('menuitem', { name: /Renommer/ }))
		const input = await screen.findByLabelText('Nom du nœud')
		await user.clear(input)
		await user.type(input, 'Facturer le client')
		await user.click(screen.getByRole('button', { name: 'Renommer' }))
		await clickSave()

		await waitFor(() => {
			expect(
				api.calls.some(
					(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
				),
			).toBe(true)
		})
		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		const body = (call?.params as { body: Schemas.SaveWorkflowVersionRequest })
			.body
		expect(body.graph.connectors).toEqual([
			{ ...saved, name: 'Facturer le client' },
		])
		expect(
			window.localStorage.getItem('mestier:automation:node-meta:workflow-1'),
		).toBeNull()
	})
})

describe('WorkflowPageFeature (editor) — a node made from a collection operation', () => {
	const COLLECTION_PATH =
		'/api/v1/organizations/{organization_id}/automation/collections/{collection_id}'
	const PETS_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'

	const OPERATION_DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
		...descriptor('collection.operation', "Opération d'une collection", {
			Optional: ['bearer_token'],
		}),
		family: 'http',
	}

	const PETS: Schemas.CollectionResponse = {
		id: PETS_ID,
		organization_id: 'org-1',
		slug: 'petstore',
		title: 'Petstore',
		version: '2.0',
		description: '',
		base_url: 'https://pets.test',
		source: { kind: 'url', url: 'https://pets.test/openapi.json' },
		imported_at: '2026-11-06T09:30:00Z',
		security: [],
		operations: [
			{
				id: 'getPet',
				method: 'GET',
				path: '/pets/{id}',
				summary: 'Lire un animal',
				tag: 'pets',
				parameters: [
					{
						name: 'id',
						location: 'path',
						required: true,
						description: '',
						schema: { type: 'string' },
					},
				],
				request_body: null,
				response: null,
				response_status: '200',
			},
		],
		schemas: {},
		history: [],
	}

	const OPERATION_NODE: Schemas.PlacedConnectorDto = {
		id: 'c1',
		kind: 'collection.operation',
		version: 1,
		name: 'Lire un animal',
		description: 'GET /pets/{id}',
		config: { path: { id: '5' } },
		operation: { collection_id: PETS_ID, operation_id: 'getPet' },
	}

	function renderOperationWorkflow(collection: (params: unknown) => unknown) {
		return renderFeature((fakeApi) => {
			fakeApi.mockGet(CONNECTORS_PATH, () => ({
				data: { auth_schemes: [], connectors: [OPERATION_DESCRIPTOR] },
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: { connectors: [OPERATION_NODE], edges: [], triggers: [] },
					layout: { c1: { x: 0, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockGet(COLLECTION_PATH, collection)
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
	}

	it('reads the collection it calls from the API and sends the reference back untouched', async () => {
		const user = userEvent.setup()
		const api = renderOperationWorkflow(() => ({
			data: PETS,
			pagination: null,
		}))
		const node = await screen.findByTestId('rf__node-c1')

		fireEvent.contextMenu(node, { clientX: 200, clientY: 150 })
		await user.click(await screen.findByRole('menuitem', { name: /Renommer/ }))
		const input = await screen.findByLabelText('Nom du nœud')
		await user.clear(input)
		await user.type(input, 'Lire la fiche')
		await user.click(screen.getByRole('button', { name: 'Renommer' }))
		await clickSave()

		await waitFor(() => {
			expect(
				api.calls.some(
					(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
				),
			).toBe(true)
		})
		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		const body = (call?.params as { body: Schemas.SaveWorkflowVersionRequest })
			.body
		expect(body.graph.connectors).toEqual([
			{ ...OPERATION_NODE, name: 'Lire la fiche' },
		])
		expect(
			api.calls.filter((c) => c.path === COLLECTION_PATH).map((c) => c.params),
		).toEqual([{ path: { organization_id: 'org-1', collection_id: PETS_ID } }])
		expect(
			Object.keys(window.localStorage).filter((key) =>
				/node-meta|collections/.test(key),
			),
		).toEqual([])
	})

	it('badges the node once the API says the collection is gone', async () => {
		renderOperationWorkflow(() => {
			throw Object.assign(new Error('Resource not found'), { status: 404 })
		})

		const node = await screen.findByTestId('rf__node-c1')

		expect(
			await within(node).findByRole('img', {
				name: 'À corriger : La collection de cette opération a été supprimée.',
			}),
		).toBeDefined()
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

describe('WorkflowPageFeature (editor) — the refusal of a reference to a node', () => {
	it('tells in French, on the node and on the field, that a node an expression reads does not run before it', async () => {
		const withField = {
			...descriptor('test.withfield', 'Avec un champ'),
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
		const graph: Schemas.GraphDto = {
			connectors: [
				{
					id: 'c0',
					kind: 'test.withfield',
					version: 1,
					config: {},
					name: 'Tarifs',
				},
				{
					id: 'c1',
					kind: 'test.withfield',
					version: 1,
					config: { url: "{{ $('Tarifs').item.json.url }}" },
				},
			],
			edges: [{ from: 'c0', to: 'c1', branch: null }],
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
					layout: { c0: { x: 0, y: 0 }, c1: { x: 280, y: 0 } },
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('Invalid graph'), {
					status: 422,
					data: {
						details: {
							errors: [
								{
									connector_id: 'c1',
									field: 'url',
									message:
										'connector `c1` field `url` references node `Tarifs`, which does not run before it',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()
		const node = await screen.findByTestId('rf__node-c1')
		await within(node).findByLabelText(
			'À corriger : Le nœud « Tarifs » n’est pas relié en amont de ce nœud.',
		)

		fireEvent.doubleClick(node)
		const dialog = await screen.findByRole('dialog')
		expect(
			within(dialog).getAllByText(
				'Le nœud « Tarifs » n’est pas relié en amont de ce nœud.',
			).length,
		).toBeGreaterThan(0)
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

	it('can be tested before it exists, on the graph as it stands, without creating anything', async () => {
		const user = userEvent.setup()
		const rendered = renderPage((api) => {
			mockCreation(api)
			api.mockMutation('post', WORKFLOW_DRY_RUN_PATH, () => ({
				data: { steps: [] },
				pagination: null,
			}))
		}, DRAFT_URL)
		await addClickTrigger(user)

		const button = screen.getByRole('button', {
			name: 'Tester le workflow',
		}) as HTMLButtonElement
		expect(button.disabled).toBe(false)
		expect(button.getAttribute('title')).toBeNull()
		await user.click(button)

		await waitFor(() => {
			expect(
				mutationCalls(rendered.api).map(
					(call) => `${call.method} ${call.path}`,
				),
			).toEqual([`post ${WORKFLOW_DRY_RUN_PATH}`])
		})
		expect(mutationCalls(rendered.api)[0].params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: { graph: { triggers: [{ id: 't1', kind: 'Manual' }] } },
		})
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

describe('WorkflowPageFeature (editor) — testing the workflow', () => {
	const TESTED_GRAPH: Schemas.GraphDto = {
		connectors: [connector('c1'), connector('c2')],
		edges: [
			{ from: 't1', to: 'c1', branch: null },
			{ from: 'c1', to: 'c2', branch: null },
		],
		triggers: [{ id: 't1', kind: 'Manual' }],
	}
	const TESTED_LAYOUT = {
		t1: { x: -220, y: 0 },
		c1: { x: 0, y: 0 },
		c2: { x: 280, y: 0 },
	}

	function dryStep(
		connectorId: string,
		patch: Partial<Schemas.DryRunStepResponse> = {},
	): Schemas.DryRunStepResponse {
		return {
			connector_id: connectorId,
			iteration_path: '',
			status: 'succeeded',
			input: null,
			output: null,
			error: null,
			...patch,
		}
	}

	function dryRunCalls(api: FakeApi) {
		return api.calls.filter((call) => call.path === WORKFLOW_DRY_RUN_PATH)
	}

	function renderTested(
		respond: (params: unknown) => unknown,
		configure: (api: FakeApi) => void = () => {},
		graph: Schemas.GraphDto = TESTED_GRAPH,
	) {
		return renderFeature((api) => {
			api.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout: TESTED_LAYOUT }),
				pagination: null,
			}))
			api.mockMutation('post', WORKFLOW_DRY_RUN_PATH, respond)
			configure(api)
		})
	}

	function answering(...steps: Schemas.DryRunStepResponse[]) {
		return () => ({ data: { steps }, pagination: null })
	}

	function refusal(status: number, message: string, errors?: unknown) {
		return Object.assign(new Error(message), {
			status,
			data: errors === undefined ? undefined : { details: { errors } },
		})
	}

	async function clickTest(user: ReturnType<typeof userEvent.setup>) {
		await user.click(
			await screen.findByRole('button', { name: 'Tester le workflow' }),
		)
	}

	async function elapse(ms: number) {
		await act(async () => {
			await vi.advanceTimersByTimeAsync(ms)
		})
	}

	function badgeOn(nodeId: string, label: string) {
		return within(screen.getByTestId(`rf__node-${nodeId}`)).queryByRole('img', {
			name: label,
		})
	}

	const SUCCEEDED = 'Exécuté avec succès'
	const FAILED = 'Exécution en échec'

	describe('with real timers', () => {
		it('runs the graph as it stands in the editor, unsaved edits included, and saves nothing', async () => {
			const user = userEvent.setup()
			const api = renderTested(answering(dryStep('c1')))

			const node = await screen.findByTestId('rf__node-c2')
			fireEvent.contextMenu(node, { clientX: 200, clientY: 150 })
			await user.click(
				await screen.findByRole('menuitem', { name: /Renommer/ }),
			)
			const input = await screen.findByLabelText('Nom du nœud')
			await user.clear(input)
			await user.type(input, 'Facturer')
			await user.click(screen.getByRole('button', { name: 'Renommer' }))
			expect(screen.getByText('Modifications non enregistrées')).toBeDefined()

			await clickTest(user)

			await waitFor(() => {
				expect(dryRunCalls(api)).toHaveLength(1)
			})
			expect(dryRunCalls(api)[0].params).toEqual({
				path: { organization_id: 'org-1' },
				body: {
					graph: {
						...TESTED_GRAPH,
						connectors: [
							connector('c1'),
							{ ...connector('c2'), name: 'Facturer' },
						],
					},
				},
			})
			expect(mutationCalls(api).map((call) => call.method)).toEqual(['post'])
			expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		})

		it('runs a saved workflow as saved when nothing was touched', async () => {
			const user = userEvent.setup()
			const api = renderTested(answering())

			await clickTest(user)

			await waitFor(() => {
				expect(dryRunCalls(api)).toHaveLength(1)
			})
			expect(dryRunCalls(api)[0].params).toMatchObject({
				body: { graph: TESTED_GRAPH },
			})
		})

		it('writes nothing to the journal of executions: the runs are not read again', async () => {
			const user = userEvent.setup()
			const api = renderTested(answering(dryStep('c1')))
			await screen.findByTestId('rf__node-c1')
			const runReads = () =>
				api.calls.filter((call) => call.path === RUNS_PATH).length
			const before = runReads()

			await clickTest(user)
			await waitFor(() => {
				expect(dryRunCalls(api)).toHaveLength(1)
			})

			expect(runReads()).toBe(before)
		})

		it('does not offer the test to somebody who cannot manage the automation', async () => {
			renderTested(answering(), (api) => {
				api.mockGet(MY_PERMISSIONS_PATH, () => ({
					data: { permissions: ['VIEW_AUTOMATION'] },
					pagination: null,
				}))
			})

			await screen.findByTestId('rf__node-c1')
			await waitFor(() => {
				expect(
					screen.queryByRole('button', { name: 'Tester le workflow' }),
				).toBeNull()
			})
			fireEvent.doubleClick(screen.getByTestId('rf__node-c2'))
			const dialog = await screen.findByRole('dialog')
			fireEvent.click(within(dialog).getByRole('radio', { name: 'Exécution' }))
			expect(
				within(dialog).queryByRole('button', { name: 'Tester le workflow' }),
			).toBeNull()
		})

		it('badges the nodes named by a 422 and banners the graph errors, like a save', async () => {
			const user = userEvent.setup()
			renderTested(() => {
				throw refusal(422, 'Invalid graph', [
					{
						connector_id: 'c1',
						field: null,
						message: 'Identification manquante',
					},
					{
						connector_id: null,
						field: null,
						message: 'Le graphe contient un cycle',
					},
				])
			})

			await clickTest(user)

			const node = await screen.findByTestId('rf__node-c1')
			expect(
				await within(node).findByLabelText(
					'À corriger : Identification manquante',
				),
			).toBeDefined()
			expect(
				await screen.findByText('Le graphe contient un cycle'),
			).toBeDefined()
			expect(
				(
					screen.getByRole('button', {
						name: 'Tester le workflow',
					}) as HTMLButtonElement
				).disabled,
			).toBe(false)
		})

		it('lifts the badge of a node a test refused once it is corrected', async () => {
			const user = userEvent.setup()
			renderTested(() => {
				throw refusal(422, 'Invalid graph', [
					{
						connector_id: 'c1',
						field: null,
						message: 'Identification manquante',
					},
					{ connector_id: 'c2', field: null, message: 'Autre souci' },
				])
			})
			await clickTest(user)
			const first = await screen.findByTestId('rf__node-c1')
			await within(first).findByLabelText(
				'À corriger : Identification manquante',
			)

			fireEvent.contextMenu(first, { clientX: 200, clientY: 150 })
			await user.click(
				await screen.findByRole('menuitem', { name: /Renommer/ }),
			)
			const input = await screen.findByLabelText('Nom du nœud')
			await user.clear(input)
			await user.type(input, 'Appeler')
			await user.click(screen.getByRole('button', { name: 'Renommer' }))

			await waitFor(() => {
				expect(
					within(screen.getByTestId('rf__node-c1')).queryByLabelText(
						/À corriger/,
					),
				).toBeNull()
			})
			expect(
				within(screen.getByTestId('rf__node-c2')).getByLabelText(
					'À corriger : Autre souci',
				),
			).toBeDefined()
		})

		it.each([
			[400, 'Failed to parse the request body as JSON'],
			[403, 'Forbidden'],
			[413, 'Payload Too Large'],
			[422, 'trigger `ghost` is not in this graph'],
			[503, 'Service Unavailable'],
		])('says a %i that carries no graph errors below the canvas', async (status, message) => {
			const user = userEvent.setup()
			renderTested(() => {
				throw refusal(status, message)
			})

			await clickTest(user)

			const alert = await screen.findByRole('alert')
			expect(alert.textContent).toBe(
				`Le test du workflow a échoué : ${message}`,
			)
			expect(
				within(screen.getByTestId('rf__node-c1')).queryByLabelText(
					/À corriger/,
				),
			).toBeNull()
		})

		it('lets the error go with the next test', async () => {
			const user = userEvent.setup()
			let attempts = 0
			renderTested(() => {
				attempts += 1
				if (attempts === 1) throw refusal(503, 'Service Unavailable')
				return { data: { steps: [] }, pagination: null }
			})
			await clickTest(user)
			await screen.findByRole('alert')

			await clickTest(user)

			await waitFor(() => {
				expect(screen.queryByRole('alert')).toBeNull()
			})
		})
	})

	describe('with a clock we hold', () => {
		beforeEach(() => {
			vi.useFakeTimers({ shouldAdvanceTime: true })
		})

		afterEach(() => {
			vi.useRealTimers()
		})

		function setupUser() {
			return userEvent.setup({ advanceTimers: vi.advanceTimersByTime })
		}

		it('clears the badges at the answer, then reveals the nodes one by one in the order of the answer', async () => {
			const user = setupUser()
			renderTested(
				answering(
					dryStep('c1', { output: { id: 1 } }),
					dryStep('c2', { status: 'failed', error: 'URL introuvable' }),
				),
				(api) => {
					api.mockGet(RUNS_PATH, () => ({
						data: [
							{
								id: 'run-1',
								organization_id: 'org-1',
								workflow_id: 'workflow-1',
								workflow_version_id: 'v1',
								status: 'succeeded',
								created_at: '2026-08-05T00:00:00Z',
							},
						],
						pagination: null,
					}))
					api.mockGet(RUN_PATH, () => ({
						data: {
							id: 'run-1',
							organization_id: 'org-1',
							workflow_id: 'workflow-1',
							workflow_version_id: 'v1',
							status: 'succeeded',
							created_at: '2026-08-05T00:00:00Z',
							steps: [
								{
									id: 's1',
									connector_id: 'c1',
									iteration_path: '',
									attempts: 1,
									status: 'succeeded',
									created_at: '2026-08-05T00:00:01Z',
									output: { id: 9 },
								},
							],
						},
						pagination: null,
					}))
				},
			)
			await screen.findByTestId('rf__node-c1')
			await waitFor(() => {
				expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			})

			await clickTest(user)

			const running = await screen.findByRole('button', {
				name: 'Exécution en cours…',
			})
			expect((running as HTMLButtonElement).disabled).toBe(true)
			await waitFor(() => {
				expect(badgeOn('c1', SUCCEEDED)).toBeNull()
			})
			expect(badgeOn('c2', FAILED)).toBeNull()

			await elapse(500)
			expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			expect(badgeOn('c2', FAILED)).toBeNull()

			await elapse(350)
			expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			expect(badgeOn('c2', FAILED)).not.toBeNull()
			expect(
				screen.getByRole('button', { name: 'Exécution en cours…' }),
			).toBeDefined()

			await elapse(350)
			expect(
				(
					screen.getByRole('button', {
						name: 'Tester le workflow',
					}) as HTMLButtonElement
				).disabled,
			).toBe(false)
		})

		it('gives no badge to a node the test did not take, and says so in its details', async () => {
			const user = setupUser()
			renderTested(
				answering(dryStep('c1'), dryStep('c2', { status: 'skipped' })),
			)

			await clickTest(user)
			await elapse(1500)

			expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			expect(badgeOn('c2', SUCCEEDED)).toBeNull()
			expect(badgeOn('c2', FAILED)).toBeNull()
			fireEvent.doubleClick(screen.getByTestId('rf__node-c2'))
			const dialog = await screen.findByRole('dialog')
			expect(
				within(
					within(dialog).getByRole('region', { name: 'Sortie' }),
				).getByText('Ce nœud n’a pas été exécuté'),
			).toBeDefined()
		})

		it('updates a node at every pass of a loop and ends on the last one, which the details tell by its number', async () => {
			const user = setupUser()
			renderTested(
				answering(
					dryStep('c1'),
					dryStep('c2', { iteration_path: 'c1[0]', output: { n: 'premier' } }),
					dryStep('c2', { iteration_path: 'c1[1]', output: { n: 'deuxieme' } }),
					dryStep('c2', { iteration_path: 'c1[2]', output: { n: 'dernier' } }),
				),
			)

			await clickTest(user)
			await elapse(500)
			expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			expect(badgeOn('c2', SUCCEEDED)).toBeNull()

			await elapse(350)
			expect(badgeOn('c2', SUCCEEDED)).not.toBeNull()
			await elapse(1000)

			fireEvent.doubleClick(screen.getByTestId('rf__node-c2'))
			const dialog = await screen.findByRole('dialog')
			const output = within(dialog).getByRole('region', { name: 'Sortie' })
			expect(output.textContent).toContain('dernier')
			expect(within(output).getByText('Passage 3 sur 3')).toBeDefined()
		})

		it('stops at the first failure: that node is ✗, the nodes left unreached have no badge and say so', async () => {
			const user = setupUser()
			renderTested(
				answering(
					dryStep('c1', { status: 'failed', error: 'URL introuvable' }),
					dryStep('c2', { status: 'skipped' }),
				),
			)

			await clickTest(user)
			await elapse(500)

			expect(badgeOn('c1', FAILED)).not.toBeNull()
			expect(badgeOn('c2', SUCCEEDED)).toBeNull()
			expect(badgeOn('c2', FAILED)).toBeNull()
			await elapse(700)
			expect(
				(
					screen.getByRole('button', {
						name: 'Tester le workflow',
					}) as HTMLButtonElement
				).disabled,
			).toBe(false)

			fireEvent.doubleClick(screen.getByTestId('rf__node-c1'))
			let dialog = await screen.findByRole('dialog')
			expect(
				within(
					within(dialog).getByRole('region', { name: 'Sortie' }),
				).getByText('URL introuvable'),
			).toBeDefined()
			await user.click(
				screen.getByRole('button', { name: 'Fermer les détails du nœud' }),
			)
			await waitFor(() => {
				expect(screen.queryByRole('dialog')).toBeNull()
			})

			fireEvent.doubleClick(screen.getByTestId('rf__node-c2'))
			dialog = await screen.findByRole('dialog')
			await user.click(within(dialog).getByRole('radio', { name: 'Exécution' }))
			expect(
				within(
					within(dialog).getByRole('region', { name: 'Entrée' }),
				).getByText('Ce nœud n’a pas été atteint'),
			).toBeDefined()
		})

		it('ends on the failure of a pass of a loop: the step at fault is ✗', async () => {
			const user = setupUser()
			renderTested(
				answering(
					dryStep('c1'),
					dryStep('c2', { iteration_path: 'c1[0]', output: 1 }),
					dryStep('c2', {
						iteration_path: 'c1[1]',
						status: 'failed',
						error: 'boom',
					}),
				),
			)

			await clickTest(user)
			await elapse(1500)

			expect(badgeOn('c2', FAILED)).not.toBeNull()
			fireEvent.doubleClick(screen.getByTestId('rf__node-c2'))
			const dialog = await screen.findByRole('dialog')
			const output = within(dialog).getByRole('region', { name: 'Sortie' })
			expect(within(output).getByText('boom')).toBeDefined()
			expect(within(output).getByText('Passage 2 sur 2')).toBeDefined()
		})

		it('fills the details of a node left open, in place, as the test reaches it', async () => {
			const user = setupUser()
			renderTested(
				answering(
					dryStep('c1', {
						input: { url: 'https://un.test' },
						output: { a: 1 },
					}),
					dryStep('c2', {
						input: { url: 'https://deux.test' },
						output: { b: 'deuxieme-sortie' },
					}),
				),
			)
			fireEvent.doubleClick(await screen.findByTestId('rf__node-c2'))
			const dialog = await screen.findByRole('dialog')
			await user.click(within(dialog).getByRole('radio', { name: 'Exécution' }))
			expect(
				within(dialog).getByText('Aucune exécution pour le moment'),
			).toBeDefined()

			await user.click(
				within(dialog).getByRole('button', { name: 'Tester le workflow' }),
			)
			await waitFor(() => {
				expect(
					within(
						within(dialog).getByRole('region', { name: 'Entrée' }),
					).getByText('En attente du test'),
				).toBeDefined()
			})
			await elapse(500)
			expect(
				within(
					within(dialog).getByRole('region', { name: 'Entrée' }),
				).getByText('En attente du test'),
			).toBeDefined()

			await elapse(350)
			expect(
				within(dialog).getByRole('region', { name: 'Entrée' }).textContent,
			).toContain('https://deux.test')
			expect(
				within(dialog).getByRole('region', { name: 'Sortie' }).textContent,
			).toContain('deuxieme-sortie')
		})

		it('keeps the results through a move of the canvas and drops them with a change of the graph, giving the real run back', async () => {
			const user = setupUser()
			renderTested(
				answering(dryStep('c1', { status: 'failed', error: 'boom' })),
				(api) => {
					api.mockGet(RUNS_PATH, () => ({
						data: [
							{
								id: 'run-1',
								organization_id: 'org-1',
								workflow_id: 'workflow-1',
								workflow_version_id: 'v1',
								status: 'succeeded',
								created_at: '2026-08-05T00:00:00Z',
							},
						],
						pagination: null,
					}))
					api.mockGet(RUN_PATH, () => ({
						data: {
							id: 'run-1',
							organization_id: 'org-1',
							workflow_id: 'workflow-1',
							workflow_version_id: 'v1',
							status: 'succeeded',
							created_at: '2026-08-05T00:00:00Z',
							steps: [
								{
									id: 's1',
									connector_id: 'c1',
									iteration_path: '',
									attempts: 1,
									status: 'succeeded',
									created_at: '2026-08-05T00:00:01Z',
									output: { id: 9 },
								},
							],
						},
						pagination: null,
					}))
				},
			)
			await waitFor(() => {
				expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
			})
			await clickTest(user)
			await elapse(1500)
			expect(badgeOn('c1', FAILED)).not.toBeNull()

			dragNodeBy(screen.getByTestId('rf__node-c1'), 40, 30)
			await elapse(100)
			expect(badgeOn('c1', FAILED)).not.toBeNull()

			const node = screen.getByTestId('rf__node-c2')
			fireEvent.contextMenu(node, { clientX: 200, clientY: 150 })
			await user.click(
				await screen.findByRole('menuitem', { name: /Renommer/ }),
			)
			const input = await screen.findByLabelText('Nom du nœud')
			await user.clear(input)
			await user.type(input, 'Facturer')
			await user.click(screen.getByRole('button', { name: 'Renommer' }))

			await waitFor(() => {
				expect(badgeOn('c1', FAILED)).toBeNull()
			})
			expect(badgeOn('c1', SUCCEEDED)).not.toBeNull()
		})
	})
})

describe('WorkflowPageFeature (editor) — the scheduler and the webhook', () => {
	const BASE: Schemas.GraphDto = {
		connectors: [connector('c1')],
		edges: [{ from: 't1', to: 'c1', branch: null }],
		triggers: [{ id: 't1', kind: 'Manual' }],
	}
	const LAYOUT = { c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 } }

	function savedBody(api: FakeApi): Schemas.SaveWorkflowVersionRequest {
		const call = api.calls.find(
			(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
		)
		return (call?.params as { body: Schemas.SaveWorkflowVersionRequest }).body
	}

	function openWith(
		graph: Schemas.GraphDto,
		layout: Schemas.WorkflowLayoutDto = LAYOUT,
		credentials: Schemas.CredentialResponse[] = [],
	) {
		return renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph, layout }),
				pagination: null,
			}))
			fakeApi.mockGet(CREDENTIALS_PATH, () => ({
				data: credentials,
				pagination: null,
			}))
			fakeApi.mockGet(CREDENTIAL_KINDS_PATH, () => ({
				data: [HTTP_HEADER],
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
	}

	function refuseWith(
		status: number,
		code: string,
		errors: Schemas.GraphErrorResponse[],
	) {
		return (fakeApi: FakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({ graph: BASE, layout: LAYOUT }),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('refused'), {
					status,
					data: { code, message: 'refused', status, details: { errors } },
				})
			})
		}
	}

	async function addTrigger(label: RegExp, nodeId: string) {
		await screen.findByTestId('rf__node-t1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 400, clientY: 400 })
		fireEvent.click(await screen.findByText('Ajouter un nœud'))
		fireEvent.click(await screen.findByText('Ajouter un déclencheur'))
		fireEvent.click(
			within(
				await screen.findByRole('complementary', { name: 'Créateur de nœuds' }),
			).getByRole('button', { name: label }),
		)
		return screen.findByTestId(`rf__node-${nodeId}`)
	}

	async function choose(
		user: ReturnType<typeof userEvent.setup>,
		combobox: string,
		option: string,
	) {
		await user.click(screen.getByRole('combobox', { name: combobox }))
		await user.click(await screen.findByRole('option', { name: option }))
	}

	it('creates a scheduler, makes it weekly on Friday at 14:30 and saves the schedule', async () => {
		const user = userEvent.setup()
		const api = openWith(BASE)

		const node = await addTrigger(/Planificateur/, 't2')
		expect(within(node).getByText('Planificateur')).toBeDefined()
		expect(within(node).getByText('tous les jours à 9:00')).toBeDefined()

		fireEvent.doubleClick(node)
		await choose(user, 'Fréquence', 'Toutes les semaines')
		await choose(user, 'Jour', 'Vendredi')
		fireEvent.change(screen.getByLabelText('Heure de déclenchement'), {
			target: { value: '14:30' },
		})
		await closeDetails()

		expect(
			within(screen.getByTestId('rf__node-t2')).getByText(
				'toutes les semaines le vendredi à 14:30',
			),
		).toBeDefined()
		await clickSave()
		await waitFor(() => expect(savedBody(api)).toBeDefined())

		expect(savedBody(api).graph.triggers).toEqual([
			{ id: 't1', kind: 'Manual' },
			{
				id: 't2',
				kind: {
					Schedule: {
						frequency: 'Weekly',
						time: '14:30',
						weekday: 'Friday',
						timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
					},
				},
			},
		])
	})

	it('creates a webhook, fills its method, its URL and its identification, and saves the webhook', async () => {
		const user = userEvent.setup()
		const api = openWith(BASE, LAYOUT, [
			credential('key-1', 'http_header', 'Clé Stripe'),
		])

		const node = await addTrigger(/Webhook/, 't2')
		expect(within(node).getByText('Webhook')).toBeDefined()
		expect(within(node).getByText('Déclencheur HTTP')).toBeDefined()

		fireEvent.doubleClick(node)
		await choose(user, 'Méthode', 'PUT')
		const path = screen.getByPlaceholderText('stripe/paiement-recu')
		fireEvent.change(path, { target: { value: 'Stripe/Paiement Reçu' } })
		fireEvent.blur(path)
		await choose(user, 'Identification', 'Clé Stripe')

		expect(
			screen.getAllByText(
				`${window.location.origin}/webhooks/stripe/paiement-recu`,
			),
		).toHaveLength(2)
		await closeDetails()
		await clickSave()
		await waitFor(() => expect(savedBody(api)).toBeDefined())

		expect(savedBody(api).graph.triggers[1]).toEqual({
			id: 't2',
			kind: {
				Webhook: {
					method: 'PUT',
					path: 'stripe/paiement-recu',
					credential_id: 'key-1',
					contract: [],
					on_invalid: 'Reject',
				},
			},
		})
	})

	it('describes the contract of a webhook and what to do with a body that breaks it, and saves both', async () => {
		const user = userEvent.setup()
		const api = openWith(BASE)

		fireEvent.doubleClick(await addTrigger(/Webhook/, 't2'))
		await user.click(screen.getByRole('button', { name: 'Ajouter un champ' }))
		fireEvent.change(screen.getByLabelText('Clé nouvelle clé'), {
			target: { value: 'montant' },
		})
		await choose(user, 'Type de montant', 'Nombre')
		fireEvent.change(screen.getByLabelText('Exemple pour montant'), {
			target: { value: '2340' },
		})
		await user.click(screen.getByRole('switch', { name: 'montant requis' }))
		await choose(
			user,
			'Requête non conforme au contrat',
			'Accepter et lancer le workflow',
		)
		await closeDetails()
		await clickSave()
		await waitFor(() => expect(savedBody(api)).toBeDefined())

		expect(savedBody(api).graph.triggers[1]).toMatchObject({
			kind: {
				Webhook: {
					contract: [
						{
							name: 'montant',
							kind: 'Number',
							required: false,
							example: '2340',
						},
					],
					on_invalid: 'Accept',
				},
			},
		})
	})

	it('badges a webhook with no path before any save, and lifts the badge once the path is right', async () => {
		openWith(BASE)

		const node = await addTrigger(/Webhook/, 't2')
		expect(
			within(node).getByLabelText(
				'À corriger : Indiquez la fin de l’URL après /webhooks/.',
			),
		).toBeDefined()

		fireEvent.doubleClick(node)
		fireEvent.change(screen.getByPlaceholderText('stripe/paiement-recu'), {
			target: { value: 'recu' },
		})
		await closeDetails()

		await waitFor(() => {
			expect(
				within(screen.getByTestId('rf__node-t2')).queryByLabelText(
					/À corriger/,
				),
			).toBeNull()
		})
	})

	it('badges both webhooks of a graph that share a path, naming the other one', async () => {
		const hook = (id: string, name: string): Schemas.PlacedTriggerDto => ({
			id,
			name,
			kind: { Webhook: { method: 'POST', path: 'paiement-recu' } },
		})
		openWith(
			{
				connectors: [connector('c1')],
				edges: [
					{ from: 't1', to: 'c1', branch: null },
					{ from: 't2', to: 'c1', branch: null },
				],
				triggers: [hook('t1', 'Stripe'), hook('t2', 'Paypal')],
			},
			{ c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 }, t2: { x: 0, y: 140 } },
		)

		expect(
			await within(await screen.findByTestId('rf__node-t1')).findByLabelText(
				'À corriger : Cette URL est déjà utilisée par « Paypal » dans ce workflow.',
			),
		).toBeDefined()
		expect(
			within(screen.getByTestId('rf__node-t2')).getByLabelText(
				'À corriger : Cette URL est déjà utilisée par « Stripe » dans ce workflow.',
			),
		).toBeDefined()
	})

	it('badges a key without a name and a scheduler whose time is not a time', async () => {
		openWith(
			{
				connectors: [connector('c1')],
				edges: [
					{ from: 't1', to: 'c1', branch: null },
					{ from: 't2', to: 'c1', branch: null },
				],
				triggers: [
					{
						id: 't1',
						kind: {
							Webhook: {
								method: 'POST',
								path: 'x',
								contract: [{ name: '', kind: 'Text', required: true }],
							},
						},
					},
					{ id: 't2', kind: { Schedule: { frequency: 'Daily', time: '9h' } } },
				],
			},
			{ c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 }, t2: { x: 0, y: 140 } },
		)

		expect(
			await within(await screen.findByTestId('rf__node-t1')).findByLabelText(
				'À corriger : Clé n° 1 : Donnez un nom à cette clé.',
			),
		).toBeDefined()
		expect(
			within(screen.getByTestId('rf__node-t2')).getByLabelText(
				'À corriger : Indiquez une heure au format HH:MM, par exemple 09:00.',
			),
		).toBeDefined()
	})

	it('puts the errors of a refused save on the webhook and on the field they name', async () => {
		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: {
						...BASE,
						triggers: [
							{
								id: 't1',
								kind: {
									Webhook: {
										method: 'POST',
										path: 'paiement-recu',
										credential_id: 'ghost',
									},
								},
							},
						],
					},
					layout: LAYOUT,
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(new Error('refused'), {
					status: 422,
					data: {
						code: 'E_GRAPH_INVALID',
						details: {
							errors: [
								{
									trigger_id: 't1',
									field: 'credential',
									message: 'trigger `t1` references unknown credential `ghost`',
								},
								{
									trigger_id: 't1',
									field: 'contract.montant',
									message:
										'trigger `t1` names the contract key `montant` more than once',
								},
							],
						},
					},
				})
			})
		})

		await clickSave()
		const node = await screen.findByTestId('rf__node-t1')
		await within(node).findByLabelText(/À corriger/)

		fireEvent.doubleClick(node)
		expect(
			await screen.findByText('Cette identification n’existe plus.'),
		).toBeDefined()
	})

	it('puts the 409 of a path another workflow holds on the path of the webhook, in French', async () => {
		renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: workflowDetail({
					graph: {
						...BASE,
						triggers: [
							{
								id: 't1',
								kind: {
									Webhook: {
										method: 'POST',
										path: 'paiement-recu',
										contract: [],
									},
								},
							},
						],
					},
					layout: LAYOUT,
				}),
				pagination: null,
			}))
			fakeApi.mockMutation('put', WORKFLOW_VERSIONS_PATH, () => {
				throw Object.assign(
					new Error('a webhook path is already used by another workflow'),
					{
						status: 409,
						data: {
							code: 'E_WEBHOOK_PATH_TAKEN',
							message: 'a webhook path is already used by another workflow',
							status: 409,
							details: {
								errors: [
									{
										trigger_id: 't1',
										field: 'path',
										message:
											'trigger `t1` uses a webhook path another workflow already holds',
									},
								],
							},
						},
					},
				)
			})
		})

		await clickSave()
		const node = await screen.findByTestId('rf__node-t1')
		expect(
			await within(node).findByLabelText(
				'À corriger : Ce chemin est déjà utilisé.',
			),
		).toBeDefined()
		expect(screen.queryByText(/L’enregistrement a échoué/)).toBeNull()

		fireEvent.doubleClick(node)
		const dialog = await screen.findByRole('dialog')
		expect(
			(
				await within(dialog).findByPlaceholderText('stripe/paiement-recu')
			).getAttribute('aria-invalid'),
		).toBe('true')
		expect(within(dialog).getByRole('alert').textContent).toBe(
			'Ce chemin est déjà utilisé.',
		)
	})

	it('banners the cap of webhooks of the organization, and badges no node', async () => {
		renderFeature(
			refuseWith(422, 'E_GRAPH_INVALID', [
				{ message: 'an organization holds at most 200 webhook routes' },
			]),
		)

		await clickSave()

		expect(
			await screen.findByText(
				'Votre organisation a atteint la limite de 200 webhooks.',
			),
		).toBeDefined()
		expect(
			within(screen.getByTestId('rf__node-t1')).queryByLabelText(/À corriger/),
		).toBeNull()
	})

	it('banners the cap of schedules of the organization', async () => {
		renderFeature(
			refuseWith(422, 'E_GRAPH_INVALID', [
				{ message: 'an organization holds at most 200 schedules' },
			]),
		)

		await clickSave()

		expect(
			await screen.findByText(
				'Votre organisation a atteint la limite de 200 planificateurs.',
			),
		).toBeDefined()
	})

	it('reads the webhook and the scheduler of a saved workflow and saves them back untouched', async () => {
		const triggers: Schemas.PlacedTriggerDto[] = [
			{
				id: 't1',
				name: 'Paiement',
				kind: {
					Webhook: {
						method: 'DELETE',
						path: 'a/b',
						credential_id: null,
						contract: [
							{ name: 'x', kind: 'List', required: true, example: '[1]' },
						],
						on_invalid: 'Accept',
					},
				},
			},
			{
				id: 't2',
				kind: {
					Schedule: {
						frequency: 'Weekly',
						time: '07:05',
						weekday: 'Sunday',
						timezone: 'UTC',
					},
				},
			},
		]
		const api = openWith(
			{
				connectors: [connector('c1')],
				edges: [
					{ from: 't1', to: 'c1', branch: null },
					{ from: 't2', to: 'c1', branch: null },
				],
				triggers,
			},
			{ c1: { x: 280, y: 0 }, t1: { x: 0, y: 0 }, t2: { x: 0, y: 140 } },
		)

		expect(
			within(await screen.findByTestId('rf__node-t2')).getByText(
				'toutes les semaines le dimanche à 7:05',
			),
		).toBeDefined()
		fireEvent.doubleClick(screen.getByTestId('rf__node-t1'))
		expect(
			(
				(await screen.findByPlaceholderText(
					'stripe/paiement-recu',
				)) as HTMLInputElement
			).value,
		).toBe('a/b')
		await closeDetails()
		fireEvent.click(screen.getByRole('button', { name: /Enregistrer/ }))
		await waitFor(() => expect(savedBody(api)).toBeDefined())

		expect(savedBody(api).graph.triggers).toEqual(triggers)
	})

	it('duplicates a webhook with its kind copied, not shared, and flags the copy that takes the same path', async () => {
		const user = userEvent.setup()
		openWith({
			connectors: [connector('c1')],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [
				{
					id: 't1',
					name: 'Stripe',
					kind: {
						Webhook: { method: 'POST', path: 'stripe', contract: [] },
					},
				},
			],
		})

		const original = await screen.findByTestId('rf__node-t1')
		fireEvent.click(original)
		await user.keyboard('{Control>}d{/Control}')

		const copy = await screen.findByTestId('rf__node-t2')
		expect(
			await within(copy).findByLabelText(
				'À corriger : Cette URL est déjà utilisée par « Stripe » dans ce workflow.',
			),
		).toBeDefined()
	})
})
