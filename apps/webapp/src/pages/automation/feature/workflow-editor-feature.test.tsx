import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Graph } from '#/hooks/use-automation'
import { WorkflowEditorFeature } from '#/pages/automation/feature/workflow-editor-feature'
import { renderWithRouter } from '#/test/render-with-router'

// jsdom has neither `ResizeObserver` (React Flow measures its viewport with
// it) nor pointer-capture/`scrollIntoView` (Radix's `Select`) — see
// `workflow-editor-ui.test.tsx`'s identical comment for why this stays
// scoped per-file rather than in `vitest.setup.ts`.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
;(globalThis as { ResizeObserver?: unknown }).ResizeObserver ??=
	ResizeObserverStub
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
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const WORKFLOW_VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const HTTP_CONNECTOR = {
	kind: 'http.request',
	label: 'Requête HTTP',
	family: 'http',
	version: 1,
	auth: 'None',
	fields: [
		{
			name: 'url',
			label: 'URL',
			kind: 'Text',
			required: true,
			expression: false,
			secret: false,
		},
	],
	output_example: { status: 200 },
}

function workflowDetail(graph: Graph = { connectors: [], edges: [] }) {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Relance devis',
		description: null,
		enabled: true,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		current_version:
			graph.connectors.length === 0 && graph.edges.length === 0
				? null
				: {
						id: 'version-1',
						workflow_id: 'workflow-1',
						version: 1,
						graph,
					},
	}
}

interface FakeApiHandlers {
	workflow?: unknown
	saveVersion?: (params: unknown) => unknown
	saveVersionError?: unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const workflow = handlers.workflow ?? workflowDetail()

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
						if (path === CONNECTORS_PATH) {
							return {
								data: { auth_schemes: [], connectors: [HTTP_CONNECTOR] },
								pagination: null,
							}
						}
						if (path === WORKFLOW_PATH) {
							return { data: workflow, pagination: null }
						}
						if (path === CREDENTIALS_PATH) {
							return { data: [], pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						if (method === 'put' && path === WORKFLOW_VERSIONS_PATH) {
							if (handlers.saveVersionError) throw handlers.saveVersionError
							if (!handlers.saveVersion) {
								throw new Error('saveVersion not mocked')
							}
							return handlers.saveVersion(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return calls
}

function renderFeature(handlers: FakeApiHandlers = {}) {
	const calls = installFakeTanstackApi(handlers)
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

	return renderWithRouter(
		<Providers>
			<WorkflowEditorFeature workflowId="workflow-1" />
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

describe('WorkflowEditorFeature — loading the graph', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the workflow name once loaded', async () => {
		await renderFeature()

		expect(await screen.findByText('Relance devis')).toBeDefined()
	})
})

describe('WorkflowEditorFeature — adding and saving a connector', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('adds a connector from the palette and saves it in the graph', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			saveVersion: (params) => ({
				data: {
					id: 'version-2',
					workflow_id: 'workflow-1',
					version: 2,
					graph: paramsBody(params).graph,
				},
			}),
		})

		await screen.findByText('Relance devis')
		await user.click(screen.getByText('Requête HTTP'))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			const saveCall = calls.find(
				(c) => c.method === 'put' && c.path === WORKFLOW_VERSIONS_PATH,
			)
			expect(saveCall).toBeDefined()
			const graph = paramsBody(saveCall?.params).graph as {
				connectors: Array<{ kind: string }>
			}
			expect(graph.connectors).toHaveLength(1)
			expect(graph.connectors[0].kind).toBe('http.request')
		})
	})
})

describe('WorkflowEditorFeature — editing a placed connector', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('selecting a node opens its fields, bound to the graph config', async () => {
		await renderFeature({
			workflow: workflowDetail({
				connectors: [
					{ id: 'c1', kind: 'http.request', version: 1, config: {} },
				],
				edges: [],
			}),
		})

		await screen.findByText('Relance devis')
		fireEvent.click(screen.getAllByText('Requête HTTP')[1])

		expect(await screen.findByLabelText('URL')).toBeDefined()
	})
})

describe('WorkflowEditorFeature — structured save error', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('maps a 422 GraphInvalid onto the failing field, not a generic banner', async () => {
		const user = userEvent.setup()
		await renderFeature({
			workflow: workflowDetail({
				connectors: [
					{ id: 'c1', kind: 'http.request', version: 1, config: {} },
				],
				edges: [],
			}),
			saveVersionError: {
				response: {
					status: 422,
					data: {
						code: 'graph_invalid',
						message: 'Graphe invalide',
						status: 422,
						details: {
							errors: [
								{ connector_id: 'c1', field: 'url', message: 'URL requise' },
							],
						},
					},
				},
			},
		})

		await screen.findByText('Relance devis')
		fireEvent.click(screen.getAllByText('Requête HTTP')[1])
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(await screen.findByText('URL requise')).toBeDefined()
	})
})

function paramsBody(params: unknown) {
	return (params as { body?: Record<string, unknown> }).body ?? {}
}
