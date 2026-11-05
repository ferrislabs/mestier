import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkflowExecutionsFeature } from '#/pages/automation/feature/workflow-executions-feature'
import {
	type FakeApi,
	installFakeTanstackApi,
} from '#/pages/automation/test/fake-tanstack-api'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'
import { renderWithRouter } from '#/test/render-with-router'

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
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
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const GRAPH: Schemas.GraphDto = {
	connectors: [{ id: 'c1', kind: 'test.simple', version: 1, config: {} }],
	edges: [],
	triggers: [],
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

function run(overrides: Record<string, unknown> = {}) {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-05T00:00:00Z',
		started_at: '2026-08-05T00:00:00.000Z',
		finished_at: '2026-08-05T00:00:01.000Z',
		trigger_event_id: null,
		...overrides,
	}
}

function runDetail(overrides: Record<string, unknown> = {}) {
	return {
		...run(),
		graph: GRAPH,
		steps: [
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
		...overrides,
	}
}

function render(
	configure: (api: FakeApi) => void = () => {},
	permissions: string[] = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
	runId?: string,
) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOW_PATH, () => ({
		data: {
			id: 'workflow-1',
			organization_id: 'org-1',
			name: 'Créer une facture Odoo',
			description: null,
			enabled: true,
			layout: null,
			current_version: null,
			created_at: '2026-08-01T00:00:00Z',
			updated_at: '2026-08-01T00:00:00Z',
		},
		pagination: null,
	}))
	api.mockGet(CONNECTORS_PATH, () => ({
		data: {
			auth_schemes: [],
			connectors: [descriptor('test.simple', 'Étape simple')],
		},
		pagination: null,
	}))
	api.mockGet(EVENTS_PATH, () => ({ data: [], pagination: null }))
	api.mockGet(RUNS_PATH, () => ({ data: [run()], pagination: null }))
	api.mockGet(RUN_PATH, () => ({ data: runDetail(), pagination: null }))
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions },
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

	const rendered = renderWithRouter(
		<Providers>
			<WorkflowExecutionsFeature workflowId="workflow-1" runId={runId} />
		</Providers>,
	)

	return { api, rendered }
}

beforeEach(() => {
	installFlowTestEnvironment()
})

async function openStep(name: string) {
	const steps = await screen.findByRole('complementary', {
		name: 'Déroulé de l’exécution',
	})
	await userEvent.click(
		within(steps).getByRole('button', { name: new RegExp(name) }),
	)
}

describe('WorkflowExecutionsFeature — the list', () => {
	it('lists only this workflow’s runs, most recent first', async () => {
		render((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					run({ id: 'aaaaaaaa-old', created_at: '2026-08-01T00:00:00Z' }),
					run({ id: 'bbbbbbbb-new', created_at: '2026-08-03T00:00:00Z' }),
					run({ id: 'cccccccc-other', workflow_id: 'workflow-2' }),
				],
				pagination: null,
			}))
		})

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		const items = within(list).getAllByRole('button')
		expect(items.map((item) => item.textContent)).toEqual([
			expect.stringContaining('#bbbbbbbb'),
			expect.stringContaining('#aaaaaaaa'),
		])
	})

	it('selects the given run', async () => {
		render(
			(api) => {
				api.mockGet(RUNS_PATH, () => ({
					data: [
						run({ id: 'aaaaaaaa-old', created_at: '2026-08-01T00:00:00Z' }),
						run({ id: 'bbbbbbbb-new', created_at: '2026-08-03T00:00:00Z' }),
					],
					pagination: null,
				}))
				api.mockGet(RUN_PATH, (params) => ({
					data: runDetail({
						id: (params as { path: { run_id: string } }).path.run_id,
					}),
					pagination: null,
				}))
			},
			undefined,
			'aaaaaaaa-old',
		)

		expect(
			await screen.findByRole('region', { name: 'Exécution #aaaaaaaa' }),
		).toBeDefined()
	})

	it('says there has never been a run', async () => {
		render((api) => {
			api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
		})

		expect(
			await screen.findByText('Aucune exécution pour le moment'),
		).toBeDefined()
	})

	it('reports a failed load of the runs', async () => {
		render((api) => {
			api.mockGet(RUNS_PATH, () => {
				throw new Error('boom')
			})
		})

		expect(await screen.findByText('boom')).toBeDefined()
	})

	it('navigates to a run when it is picked in the list', async () => {
		const user = userEvent.setup()
		const { rendered } = render((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					run({ id: 'aaaaaaaa-old', created_at: '2026-08-01T00:00:00Z' }),
					run({ id: 'bbbbbbbb-new', created_at: '2026-08-03T00:00:00Z' }),
				],
				pagination: null,
			}))
		})

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await user.click(within(list).getByRole('button', { name: /#aaaaaaaa/ }))

		const { router } = await rendered
		await waitFor(() =>
			expect(router.state.location.pathname).toBe(
				'/o/atelier-bois/automatisation/workflow-1/executions/aaaaaaaa-old',
			),
		)
	})
})

describe('WorkflowExecutionsFeature — the run', () => {
	it('draws the pinned graph and lists the step', async () => {
		render()

		const steps = await screen.findByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText('Étape simple')).toBeDefined()
		expect(screen.getAllByText('Étape simple').length).toBeGreaterThan(1)
	})

	it('shows a notice instead of a canvas when the pinned graph is gone', async () => {
		render((api) => {
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({ graph: null }),
				pagination: null,
			}))
		})

		expect(
			await screen.findByText(/graphe n’est plus disponible/),
		).toBeDefined()
	})

	it('reports a failed load of the run rather than a blank pane', async () => {
		render((api) => {
			api.mockGet(RUN_PATH, () => {
				throw new Error('run not found')
			})
		})

		expect(
			await screen.findByText(/Impossible de charger cette exécution/),
		).toBeDefined()
	})

	it('surfaces the failed step in an alert', async () => {
		render((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [run({ status: 'failed' })],
				pagination: null,
			}))
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({
					status: 'failed',
					steps: [
						{
							id: 's1',
							connector_id: 'c1',
							iteration_path: '',
							attempts: 1,
							status: 'failed',
							created_at: '2026-08-05T00:00:01Z',
							error: 'SMTP expiré',
						},
					],
				}),
				pagination: null,
			}))
		})

		const alert = await screen.findByRole('alert')
		expect(alert.textContent).toContain('Échec sur « Étape simple »')
		expect(alert.textContent).toContain('SMTP expiré')
	})
})

describe('WorkflowExecutionsFeature — a node', () => {
	it('opens the node details with its input and output', async () => {
		render()

		await openStep('Étape simple')

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByText('Avant l’exécution')).toBeDefined()
		expect(within(dialog).getByText('Après l’exécution')).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — replay', () => {
	it('replays through the real mutation and refetches the run', async () => {
		const user = userEvent.setup()
		const { api } = render((fakeApi) => {
			fakeApi.mockMutation('post', RUN_REPLAY_PATH, () => ({
				data: run({ status: 'running' }),
				pagination: null,
			}))
		})

		await openStep('Étape simple')
		await user.click(
			await screen.findByRole('button', { name: /Relancer depuis ici/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Relancer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'post' && c.path === RUN_REPLAY_PATH,
			)
			expect(
				(call?.params as { body: Schemas.ReplayRunRequest } | undefined)?.body,
			).toEqual({ connector_id: 'c1' })
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
		render((fakeApi) => {
			fakeApi.mockMutation('post', RUN_REPLAY_PATH, () => {
				throw Object.assign(new Error('Conflict'), { status: 409 })
			})
		})

		await openStep('Étape simple')
		await user.click(
			await screen.findByRole('button', { name: /Relancer depuis ici/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Relancer' }))

		expect(await screen.findByText('Conflict')).toBeDefined()
	})

	it('offers no replay while the run is still going', async () => {
		render((api) => {
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({ status: 'running' }),
				pagination: null,
			}))
		})

		await openStep('Étape simple')
		await screen.findByRole('dialog')

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})

	it('offers no replay to a caller without MANAGE_AUTOMATION', async () => {
		render(undefined, ['VIEW_AUTOMATION'])

		await openStep('Étape simple')
		await screen.findByRole('dialog')

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})
})
