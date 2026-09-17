import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { RunInspectorFeature } from '#/pages/automation/feature/run-inspector-feature'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const RUN_REPLAY_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

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

function descriptor(
	kind: string,
	label: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family: 'test',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

function workflowDetail(layout: Schemas.WorkflowLayoutDto | null = null) {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Créer une facture Odoo',
		description: null,
		enabled: true,
		layout,
		current_version: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
	}
}

function runDetail(
	overrides: Partial<{
		status: string
		graph: Schemas.GraphDto | null
		steps: Schemas.RunStepResponse[]
	}> = {},
) {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: overrides.status ?? 'succeeded',
		created_at: '2026-08-05T00:00:00Z',
		started_at: '2026-08-05T00:00:00.000Z',
		finished_at: '2026-08-05T00:00:01.000Z',
		trigger_event_id: null,
		graph:
			'graph' in overrides
				? overrides.graph
				: {
						connectors: [
							{ id: 'c1', kind: 'test.simple', version: 1, config: {} },
						],
						edges: [],
						triggers: [],
					},
		steps: overrides.steps ?? [
			{
				id: 's1',
				connector_id: 'c1',
				iteration_path: '',
				attempts: 1,
				status: 'succeeded',
				created_at: '2026-08-05T00:00:01Z',
				input: { url: 'https://a' },
				output: { id: 1 },
			},
		],
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
	api.mockGet(WORKFLOW_PATH, () => ({
		data: workflowDetail(),
		pagination: null,
	}))
	api.mockGet(CONNECTORS_PATH, () => ({
		data: {
			auth_schemes: [],
			connectors: [descriptor('test.simple', 'Étape simple')],
		},
		pagination: null,
	}))
	api.mockGet(RUN_PATH, () => ({ data: runDetail(), pagination: null }))
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions: ['MANAGE_AUTOMATION'] },
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
			<RunInspectorFeature workflowId="workflow-1" runId="run-1" />
		</Providers>,
	)

	return api
}

beforeEach(() => {
	installFlowTestEnvironment()
})

describe('RunInspectorFeature — loading and drawing the run', () => {
	it('draws the pinned graph and the step once everything has loaded', async () => {
		renderFeature(() => {})

		expect(await screen.findByText('Étape simple')).toBeDefined()
		expect(screen.getByText('c1')).toBeDefined()
	})

	it('shows a notice instead of a canvas when the pinned graph is gone', async () => {
		renderFeature((api) => {
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({ graph: null }),
				pagination: null,
			}))
		})

		expect(
			await screen.findByText(/graphe n’est plus disponible/),
		).toBeDefined()
		expect(screen.getByText('c1')).toBeDefined()
	})

	it('reports a failed load rather than a blank page', async () => {
		renderFeature((api) => {
			api.mockGet(RUN_PATH, () => {
				throw new Error('run not found')
			})
		})

		expect(
			await screen.findByText('Impossible de charger cette exécution'),
		).toBeDefined()
	})
})

describe('RunInspectorFeature — replay', () => {
	it('replays through the real mutation and refetches the run', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fakeApi) => {
			fakeApi.mockMutation('post', RUN_REPLAY_PATH, () => ({
				data: runDetail({ status: 'running' }),
				pagination: null,
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: /Relancer depuis ici/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Relancer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'post' && c.path === RUN_REPLAY_PATH,
			)
			expect(call).toBeDefined()
		})

		const call = api.calls.find(
			(c) => c.method === 'post' && c.path === RUN_REPLAY_PATH,
		)
		expect((call?.params as { body: Schemas.ReplayRunRequest }).body).toEqual({
			connector_id: 'c1',
		})

		await waitFor(() => {
			const runCalls = api.calls.filter(
				(c) => c.method === 'get' && c.path === RUN_PATH,
			)
			expect(runCalls.length).toBeGreaterThan(1)
		})
	})

	it('shows the replay failure', async () => {
		const user = userEvent.setup()
		renderFeature((fakeApi) => {
			fakeApi.mockMutation('post', RUN_REPLAY_PATH, () => {
				throw Object.assign(new Error('Conflict'), { status: 409 })
			})
		})

		await user.click(
			await screen.findByRole('button', { name: /Relancer depuis ici/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Relancer' }))

		expect(await screen.findByText('Conflict')).toBeDefined()
	})
})
