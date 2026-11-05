import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, screen, waitFor, within } from '@testing-library/react'
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
const EVALUATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/expressions/evaluate'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
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
	api.mockGet(CREDENTIALS_PATH, () => ({
		data: [
			{
				id: 'cred-1',
				organization_id: 'org-1',
				name: 'Compte Odoo',
				kind: 'http_basic',
				origin: 'typed',
				created_at: '2026-08-01T00:00:00Z',
				updated_at: '2026-08-01T00:00:00Z',
			},
		],
		pagination: null,
	}))
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
						<TooltipProvider>{children}</TooltipProvider>
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
	Object.defineProperty(SVGElement.prototype, 'getBBox', {
		configurable: true,
		value: () => ({ x: 0, y: 0, width: 0, height: 0 }),
	})
})

afterEach(() => {
	vi.useRealTimers()
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

	it('puts the run in the search when it is picked in the list, instead of changing page', async () => {
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
			expect(router.state.location.href).toBe(
				'/o/atelier-bois/automatisation/workflows/workflow-1?vue=executions&execution=aaaaaaaa-old',
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

describe('WorkflowExecutionsFeature — a failed node', () => {
	it('sends to the identifications from the node details', async () => {
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

		await openStep('Étape simple')

		const dialog = await screen.findByRole('dialog')
		expect(
			within(dialog)
				.getByRole('link', { name: 'Ouvrir les identifications' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/identifications')
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

describe('WorkflowExecutionsFeature — the header and the list labels', () => {
	const CURRENT_VERSION = {
		id: 'v1',
		version: 1,
		workflow_id: 'workflow-1',
		created_at: new Date(2026, 9, 28, 11, 5).toISOString(),
		graph: {
			connectors: [],
			edges: [],
			triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
		},
	}

	function withCurrentVersion(api: FakeApi) {
		api.mockGet(WORKFLOW_PATH, () => ({
			data: {
				id: 'workflow-1',
				organization_id: 'org-1',
				name: 'Créer une facture Odoo',
				description: null,
				enabled: true,
				layout: null,
				current_version: CURRENT_VERSION,
				created_at: '2026-08-01T00:00:00Z',
				updated_at: '2026-08-01T00:00:00Z',
			},
			pagination: null,
		}))
	}

	it('writes the current version as a date and flags it as current', async () => {
		render(withCurrentVersion)

		expect(
			await screen.findByText('Version du 28 oct. 2026 à 11:05'),
		).toBeDefined()
		expect(screen.getByText('· actuelle')).toBeDefined()
	})

	it('names an event-triggered run after the event of the workflow trigger', async () => {
		render((api) => {
			withCurrentVersion(api)
			api.mockGet(EVENTS_PATH, () => ({
				data: [
					{
						name: 'quote.accepted',
						label: 'Devis accepté',
						payload_example: null,
						subject_kind: 'quote',
						version: 1,
					},
				],
				pagination: null,
			}))
			api.mockGet(RUNS_PATH, () => ({
				data: [run({ trigger_event_id: 'evt-1' })],
				pagination: null,
			}))
		})

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await waitFor(() => {
			expect(within(list).getByText(/Devis accepté/)).toBeDefined()
		})
	})

	it('names a run without trigger event a manual test', async () => {
		render()

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		expect(within(list).getByText(/Test manuel/)).toBeDefined()
	})

	it('shows the not-found page when the workflow does not exist', async () => {
		render((api) => {
			api.mockGet(WORKFLOW_PATH, () => {
				throw Object.assign(new Error('not found'), { status: 404 })
			})
		})

		expect(await screen.findByText('Workflow introuvable')).toBeDefined()
	})

	it.each([
		['a 403', Object.assign(new Error('Forbidden'), { status: 403 })],
		['a 500', Object.assign(new Error('HTTP 500: boom'), { status: 500 })],
		['a network failure', new Error('Failed to fetch')],
	])('shows the error, not the not-found page, on %s', async (_name, error) => {
		render((api) => {
			api.mockGet(WORKFLOW_PATH, () => {
				throw error
			})
		})

		const alert = await screen.findByRole('alert')
		expect(alert.textContent).toContain('Impossible de charger le workflow')
		expect(alert.textContent).toContain(error.message)
		expect(screen.queryByText('Workflow introuvable')).toBeNull()
	})
})

describe('WorkflowExecutionsFeature — loops', () => {
	it('folds the iterations of a looped step under a single loop', async () => {
		render((api) => {
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({
					steps: [0, 1].map((index) => ({
						id: `s${index}`,
						connector_id: 'c1',
						iteration_path: `c1[${index}]`,
						attempts: 1,
						status: 'succeeded',
						created_at: `2026-08-05T00:00:0${index + 1}Z`,
						started_at: `2026-08-05T00:00:0${index + 1}Z`,
						output: { id: index },
					})),
				}),
				pagination: null,
			}))
		})

		expect(await screen.findByText('2 itérations')).toBeDefined()
		expect(screen.getByText('Itération 1')).toBeDefined()
		expect(screen.getByText('Itération 2')).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — node parameters', () => {
	function withExpression(api: FakeApi) {
		api.mockGet(CONNECTORS_PATH, () => ({
			data: {
				auth_schemes: [],
				connectors: [
					{
						...descriptor('test.simple', 'Étape simple'),
						fields: [
							{
								name: 'url',
								label: 'Adresse',
								kind: 'Text',
								required: true,
								secret: false,
								expression: true,
							},
						],
					},
				],
			},
			pagination: null,
		}))
		api.mockGet(RUN_PATH, () => ({
			data: runDetail({
				trigger_payload: { client: 'Hugo' },
				graph: {
					...GRAPH,
					connectors: [
						{
							id: 'c1',
							kind: 'test.simple',
							version: 1,
							config: { url: 'https://{{ trigger.client }}' },
						},
					],
				},
				steps: [
					{
						id: 's1',
						connector_id: 'c1',
						iteration_path: '',
						attempts: 1,
						status: 'succeeded',
						created_at: '2026-08-05T00:00:01Z',
						started_at: '2026-08-05T00:00:01Z',
						finished_at: '2026-08-05T00:00:02Z',
						input: [{ client: 'Hugo' }, { client: 'Nadia' }],
						output: { id: 1 },
					},
				],
			}),
			pagination: null,
		}))
		api.mockMutation('post', EVALUATE_PATH, (params) => ({
			data: {
				value: `resolved:${JSON.stringify(
					(params as { body: { context: { loop: unknown } } }).body.context
						.loop,
				)}`,
			},
		}))
	}

	it('resolves expressions through the engine for the selected input item', async () => {
		render(withExpression)

		await openStep('Étape simple')
		const dialog = await screen.findByRole('dialog')

		expect(
			await within(dialog).findByText(/resolved:\{"index":0/),
		).toBeDefined()
		expect(within(dialog).getByText('Expression')).toBeDefined()
		expect(within(dialog).getByText('1 sur 2')).toBeDefined()

		await userEvent.click(
			within(dialog).getByRole('button', { name: 'Élément suivant' }),
		)

		expect(
			await within(dialog).findByText(/resolved:\{"index":1/),
		).toBeDefined()
	})

	it('numbers the step and shows what the previous nodes produced', async () => {
		render(withExpression)

		await openStep('Étape simple')
		const dialog = await screen.findByRole('dialog')

		expect(within(dialog).getByText(/étape 1/)).toBeDefined()
		expect(
			within(dialog).getByText(/Résultats des nœuds précédents/),
		).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — the statuses of a step', () => {
	function withSteps(
		steps: Record<string, unknown>[],
		status = 'failed',
		graph: Schemas.GraphDto = GRAPH,
	) {
		return (api: FakeApi) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [run({ status })],
				pagination: null,
			}))
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({ status, steps, graph }),
				pagination: null,
			}))
		}
	}

	function stepOf(overrides: Record<string, unknown>) {
		return {
			id: 's1',
			connector_id: 'c1',
			iteration_path: '',
			attempts: 1,
			status: 'succeeded',
			created_at: '2026-08-05T00:00:01Z',
			started_at: '2026-08-05T00:00:01Z',
			...overrides,
		}
	}

	it('flags a dead step as the failure of the run, with its message', async () => {
		render(withSteps([stepOf({ status: 'dead', error: 'SMTP expiré' })]))

		const alert = await screen.findByRole('alert')
		expect(alert.textContent).toContain('Échec sur « Étape simple »')
		expect(alert.textContent).toContain('SMTP expiré')
		const steps = screen.getByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText('SMTP expiré')).toBeDefined()
	})

	it('shows an in_flight step as running', async () => {
		render(
			withSteps(
				[stepOf({ status: 'in_flight', finished_at: null })],
				'running',
			),
		)

		const steps = await screen.findByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText(/^En cours/)).toBeDefined()
		expect(screen.getByText('En cours…')).toBeDefined()
	})

	it('animates the node and the edge of a step in flight on the canvas', async () => {
		render(
			withSteps(
				[stepOf({ status: 'in_flight', finished_at: null })],
				'running',
				{
					...GRAPH,
					triggers: [{ id: 't1', kind: 'Manual' }],
					edges: [{ from: 't1', to: 'c1' }],
				},
			),
		)

		expect(
			await screen.findByRole('img', { name: 'Exécution en cours' }),
		).toBeDefined()
		expect(document.querySelector('.react-flow__edge.animated')).not.toBeNull()
	})

	it('draws no followed edge towards a skipped step', async () => {
		render(
			withSteps(
				[
					stepOf({ id: 's1', connector_id: 'c1' }),
					stepOf({
						id: 's2',
						connector_id: 'c2',
						status: 'skipped',
						attempts: 0,
					}),
				],
				'succeeded',
				{
					triggers: [{ id: 't1', kind: 'Manual' }],
					connectors: [
						{ id: 'c1', kind: 'test.simple', version: 1, config: {} },
						{ id: 'c2', kind: 'test.simple', version: 1, config: {} },
					],
					edges: [
						{ from: 't1', to: 'c1' },
						{ from: 'c1', to: 'c2' },
					],
				},
			),
		)
		await screen.findByRole('complementary', { name: 'Déroulé de l’exécution' })

		await waitFor(() =>
			expect(document.querySelectorAll('.react-flow__edge')).toHaveLength(2),
		)
		const followed = [
			...document.querySelectorAll<SVGPathElement>('.react-flow__edge-path'),
		].filter((path) => path.style.strokeDasharray === '')
		expect(followed).toHaveLength(1)
	})

	it('lists a skipped step among the steps that were not called, unnumbered', async () => {
		render(
			withSteps(
				[
					stepOf({ id: 's1', connector_id: 'c1' }),
					stepOf({
						id: 's2',
						connector_id: 'c2',
						status: 'skipped',
						attempts: 0,
					}),
				],
				'succeeded',
				{
					triggers: [],
					connectors: [
						{ id: 'c1', kind: 'test.simple', version: 1, config: {} },
						{ id: 'c2', kind: 'test.simple', version: 1, config: {} },
					],
					edges: [{ from: 'c1', to: 'c2' }],
				},
			),
		)

		const steps = await screen.findByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText('1 / 2 nœuds')).toBeDefined()
		const skipped = within(
			within(steps).getByRole('region', { name: 'Non appelés' }),
		)
		expect(skipped.getByText('Aucun élément reçu')).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — a run in progress', () => {
	const START = new Date('2026-08-05T10:00:10.000Z').getTime()

	function liveRun(overrides: Record<string, unknown> = {}) {
		return run({
			id: 'live-1',
			status: 'running',
			created_at: new Date(START - 2200).toISOString(),
			started_at: new Date(START - 2200).toISOString(),
			finished_at: null,
			...overrides,
		})
	}

	function withLiveRun(api: FakeApi) {
		api.mockGet(RUNS_PATH, () => ({ data: [liveRun()], pagination: null }))
		api.mockGet(RUN_PATH, () => ({
			data: runDetail({
				...liveRun(),
				steps: [
					{
						id: 's1',
						connector_id: 'c1',
						iteration_path: '',
						attempts: 1,
						status: 'in_flight',
						created_at: new Date(START - 1200).toISOString(),
						started_at: new Date(START - 1200).toISOString(),
					},
				],
			}),
			pagination: null,
		}))
	}

	it('counts the time since the start in the list, the header and the steps', async () => {
		vi.useFakeTimers({ toFake: ['Date'], now: START })
		render(withLiveRun)

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		expect(await within(list).findByText(/En cours depuis 2,2 s/)).toBeDefined()
		const region = await screen.findByRole('region', {
			name: /Exécution #/,
		})
		expect(within(region).getByText('· 2,2 s')).toBeDefined()
		const steps = screen.getByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText('En cours · 1,2 s')).toBeDefined()
	})

	it('keeps counting on its own between two fetches', async () => {
		vi.useFakeTimers({ toFake: ['Date'], now: START })
		render(withLiveRun)
		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await within(list).findByText(/En cours depuis 2,2 s/)

		vi.setSystemTime(START + 1000)

		expect(await within(list).findByText(/En cours depuis 3,2 s/)).toBeDefined()
		const region = screen.getByRole('region', { name: /Exécution #/ })
		expect(within(region).getByText('· 3,2 s')).toBeDefined()
		const steps = screen.getByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByText('En cours · 2,2 s')).toBeDefined()
	})

	it('counts every running run of the list, not only the selected one', async () => {
		vi.useFakeTimers({ toFake: ['Date'], now: START })
		render((api) => {
			withLiveRun(api)
			api.mockGet(RUNS_PATH, () => ({
				data: [
					liveRun(),
					liveRun({
						id: 'live-2',
						created_at: new Date(START - 9000).toISOString(),
						started_at: new Date(START - 7000).toISOString(),
					}),
				],
				pagination: null,
			}))
		})
		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await within(list).findByText(/En cours depuis 7 s/)

		vi.setSystemTime(START + 2000)

		expect(await within(list).findByText(/En cours depuis 9 s/)).toBeDefined()
		expect(within(list).getByText(/En cours depuis 4,2 s/)).toBeDefined()
	})

	it('reloads the list on its own while a run is going, so the other runs advance', async () => {
		vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval', 'Date'] })
		let listCalls = 0
		render((api) => {
			api.mockGet(RUNS_PATH, () => {
				listCalls += 1
				return {
					data: [
						run({ id: 'done-1', created_at: '2026-08-05T00:00:09Z' }),
						listCalls > 2
							? run({ id: 'live-2', created_at: '2026-08-05T00:00:01Z' })
							: run({
									id: 'live-2',
									status: 'running',
									created_at: '2026-08-05T00:00:01Z',
									finished_at: null,
								}),
					],
					pagination: null,
				}
			})
		})
		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await waitFor(() =>
			expect(within(list).getByText(/^En cours/)).toBeDefined(),
		)

		await act(async () => {
			await vi.advanceTimersByTimeAsync(3100)
		})

		await waitFor(() =>
			expect(within(list).queryByText(/^En cours/)).toBeNull(),
		)
	})
})

describe('WorkflowExecutionsFeature — the trigger of each run', () => {
	function version(id: string, event: string) {
		return {
			id,
			version: 1,
			workflow_id: 'workflow-1',
			created_at: new Date(2026, 9, 28, 11, 5).toISOString(),
			graph: {
				connectors: [],
				edges: [],
				triggers: [{ id: 't1', kind: { Events: [event] } }],
			},
		}
	}

	function withVersions(api: FakeApi) {
		api.mockGet(WORKFLOW_PATH, () => ({
			data: {
				id: 'workflow-1',
				organization_id: 'org-1',
				name: 'Créer une facture Odoo',
				description: null,
				enabled: true,
				layout: null,
				current_version: version('v2', 'quote.accepted'),
				created_at: '2026-08-01T00:00:00Z',
				updated_at: '2026-08-01T00:00:00Z',
			},
			pagination: null,
		}))
		api.mockGet(EVENTS_PATH, () => ({
			data: [
				{
					name: 'quote.accepted',
					label: 'Devis accepté',
					payload_example: null,
					subject_kind: 'quote',
					version: 1,
				},
				{
					name: 'invoice.issued',
					label: 'Facture émise',
					payload_example: null,
					subject_kind: 'invoice',
					version: 1,
				},
			],
			pagination: null,
		}))
		api.mockGet(RUNS_PATH, () => ({
			data: [
				run({
					id: 'bbbbbbbb-new',
					workflow_version_id: 'v2',
					trigger_event_id: 'evt-2',
					created_at: '2026-08-03T00:00:00Z',
				}),
				run({
					id: 'aaaaaaaa-old',
					workflow_version_id: 'v1',
					trigger_event_id: 'evt-1',
					created_at: '2026-08-01T00:00:00Z',
				}),
			],
			pagination: null,
		}))
		api.mockGet(RUN_PATH, (params) => {
			const id = (params as { path: { run_id: string } }).path.run_id
			const old = id === 'aaaaaaaa-old'
			return {
				data: runDetail({
					id,
					workflow_version_id: old ? 'v1' : 'v2',
					trigger_event_id: old ? 'evt-1' : 'evt-2',
					graph: version(
						old ? 'v1' : 'v2',
						old ? 'invoice.issued' : 'quote.accepted',
					).graph,
					steps: [],
				}),
				pagination: null,
			}
		})
	}

	it('does not give a run of an older version the trigger of the current version', async () => {
		render(withVersions)

		const list = await screen.findByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await waitFor(() =>
			expect(within(list).getAllByRole('button')[0]?.textContent).toContain(
				'Devis accepté',
			),
		)
		const [, oldest] = within(list).getAllByRole('button')
		expect(oldest?.textContent).not.toContain('Devis accepté')
		expect(oldest?.textContent).toContain('Événement')
	})

	it('names the selected older run after the trigger of its own version', async () => {
		render(withVersions, undefined, 'aaaaaaaa-old')

		const region = await screen.findByRole('region', {
			name: 'Exécution #aaaaaaaa',
		})
		await waitFor(() =>
			expect(within(region).getByText(/Facture émise/)).toBeDefined(),
		)
		const list = screen.getByRole('complementary', {
			name: 'Liste des exécutions',
		})
		await waitFor(() => {
			const [newest, oldest] = within(list).getAllByRole('button')
			expect(oldest?.textContent).toContain('Facture émise')
			expect(newest?.textContent).toContain('Devis accepté')
		})
	})
})

describe('WorkflowExecutionsFeature — the branches of the canvas', () => {
	const BRANCH_GRAPH: Schemas.GraphDto = {
		triggers: [],
		connectors: [
			{ id: 'c1', kind: 'test.condition', version: 1, config: {} },
			{ id: 'c2', kind: 'test.simple', version: 1, config: {} },
		],
		edges: [{ from: 'c1', to: 'c2', branch: 'Then' }],
	}

	function withBranches(api: FakeApi) {
		api.mockGet(CONNECTORS_PATH, () => ({
			data: {
				auth_schemes: [],
				connectors: [
					descriptor('test.simple', 'Étape simple'),
					{
						...descriptor('test.condition', 'Condition'),
						branches: ['Then', 'Else'],
					},
				],
			},
			pagination: null,
		}))
		api.mockGet(RUN_PATH, () => ({
			data: runDetail({
				graph: BRANCH_GRAPH,
				steps: [
					{
						id: 's1',
						connector_id: 'c1',
						iteration_path: '',
						attempts: 1,
						status: 'succeeded',
						created_at: '2026-08-05T00:00:01Z',
						started_at: '2026-08-05T00:00:01Z',
						output: { matched: true },
					},
					{
						id: 's2',
						connector_id: 'c2',
						iteration_path: '',
						attempts: 1,
						status: 'succeeded',
						created_at: '2026-08-05T00:00:02Z',
						started_at: '2026-08-05T00:00:02Z',
						input: {},
						output: {},
					},
				],
			}),
			pagination: null,
		}))
	}

	it('names the outputs of a condition “vrai” and “faux”', async () => {
		render(withBranches)

		expect(await screen.findByText('vrai')).toBeDefined()
		expect(screen.getByText('faux')).toBeDefined()
		expect(screen.queryByText('Then')).toBeNull()
		expect(screen.queryByText('Else')).toBeNull()
	})

	it('names the branch a node received its data from in the same words', async () => {
		render(withBranches)
		await screen.findByText('vrai')

		await openStep('Étape simple')

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByText(/\(branche vrai\)/)).toBeDefined()
		expect(within(dialog).queryByText(/branche Then/)).toBeNull()
	})
})

describe('WorkflowExecutionsFeature — what a node shows as parameters', () => {
	function withParameters(method: string) {
		return (api: FakeApi) => {
			api.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [],
					connectors: [
						{
							...descriptor('test.simple', 'Étape simple'),
							auth: { Exactly: 'http_basic' },
							fields: [
								{
									name: 'method',
									label: 'Méthode',
									kind: {
										Select: {
											options: [
												{ value: 'GET', label: 'Lecture' },
												{ value: 'POST', label: 'Envoi' },
											],
										},
									},
									required: true,
									secret: false,
									expression: false,
								},
								{
									name: 'body',
									label: 'Corps de la requête',
									kind: 'Json',
									required: false,
									secret: false,
									expression: false,
									visible_when: { field: 'method', any_of: ['POST'] },
								},
							],
						},
					],
				},
				pagination: null,
			}))
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({
					graph: {
						...GRAPH,
						connectors: [
							{
								id: 'c1',
								kind: 'test.simple',
								version: 1,
								credential_id: 'cred-1',
								config: { method, body: '{"a":1}' },
							},
						],
					},
				}),
				pagination: null,
			}))
		}
	}

	it('writes the label of the chosen option and the name of the identification', async () => {
		render(withParameters('POST'))

		await openStep('Étape simple')
		const dialog = await screen.findByRole('dialog')

		expect(await within(dialog).findByText('Envoi')).toBeDefined()
		expect(within(dialog).getByText('Compte Odoo')).toBeDefined()
		expect(within(dialog).getByText('Identification')).toBeDefined()
		expect(within(dialog).queryByText('POST')).toBeNull()
		expect(within(dialog).queryByText('cred-1')).toBeNull()
	})

	it('leaves out a field the chosen option hides', async () => {
		render(withParameters('GET'))

		await openStep('Étape simple')
		const dialog = await screen.findByRole('dialog')

		expect(await within(dialog).findByText('Lecture')).toBeDefined()
		expect(within(dialog).queryByText('Corps de la requête')).toBeNull()
	})

	it('shows a field once the chosen option reveals it', async () => {
		render(withParameters('POST'))

		await openStep('Étape simple')
		const dialog = await screen.findByRole('dialog')

		expect(await within(dialog).findByText('Corps de la requête')).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — a node that was not reached', () => {
	const TWO_NODES: Schemas.GraphDto = {
		triggers: [],
		connectors: [
			{ id: 'c1', kind: 'test.simple', version: 1, config: {} },
			{ id: 'c2', kind: 'test.second', version: 1, config: {} },
		],
		edges: [{ from: 'c1', to: 'c2' }],
	}

	function stepOf(overrides: Record<string, unknown>) {
		return {
			id: 's1',
			connector_id: 'c1',
			iteration_path: '',
			attempts: 1,
			status: 'succeeded',
			created_at: '2026-08-05T00:00:01Z',
			started_at: '2026-08-05T00:00:01Z',
			finished_at: '2026-08-05T00:00:02Z',
			input: { a: 1 },
			output: { b: 2 },
			...overrides,
		}
	}

	function withSteps(steps: Record<string, unknown>[], status: string) {
		return (api: FakeApi) => {
			api.mockGet(CONNECTORS_PATH, () => ({
				data: {
					auth_schemes: [],
					connectors: [
						descriptor('test.simple', 'Étape simple'),
						descriptor('test.second', 'Seconde étape'),
					],
				},
				pagination: null,
			}))
			api.mockGet(RUNS_PATH, () => ({
				data: [run({ status })],
				pagination: null,
			}))
			api.mockGet(RUN_PATH, () => ({
				data: runDetail({ status, graph: TWO_NODES, steps }),
				pagination: null,
			}))
		}
	}

	it('explains that a skipped step received no item, without counting its input', async () => {
		render(
			withSteps(
				[
					stepOf({}),
					stepOf({
						id: 's2',
						connector_id: 'c2',
						status: 'skipped',
						attempts: 0,
						input: null,
						output: null,
					}),
				],
				'succeeded',
			),
		)

		await openStep('Seconde étape')
		const dialog = await screen.findByRole('dialog')

		expect(
			within(dialog).getByText(
				'Non appelé : la branche qui mène à ce nœud n’a transmis aucun élément.',
			),
		).toBeDefined()
		expect(within(dialog).getByText('Entrée')).toBeDefined()
		expect(
			within(dialog).getByText(
				'Ce nœud n’a pas été exécuté pendant cette exécution.',
			),
		).toBeDefined()
	})

	it('blames the dead step for the nodes that come after it', async () => {
		render(
			withSteps(
				[stepOf({ status: 'dead', error: 'SMTP expiré', output: null })],
				'failed',
			),
		)

		await openStep('Seconde étape')
		const dialog = await screen.findByRole('dialog')

		expect(
			within(dialog).getByText(
				'Non appelé : l’exécution s’est arrêtée sur « Étape simple », avant d’atteindre ce nœud.',
			),
		).toBeDefined()
	})

	it('says a pending step has not been reached yet', async () => {
		render(
			withSteps(
				[
					stepOf({}),
					stepOf({
						id: 's2',
						connector_id: 'c2',
						status: 'pending',
						attempts: 0,
						started_at: null,
						finished_at: null,
						input: null,
						output: null,
					}),
				],
				'running',
			),
		)

		await openStep('Seconde étape')
		const dialog = await screen.findByRole('dialog')

		expect(
			within(dialog).getByText(
				'Pas encore atteint : l’exécution n’est pas encore arrivée à ce nœud.',
			),
		).toBeDefined()
		expect(
			within(dialog).getByText('Ce nœud n’a pas encore été exécuté.'),
		).toBeDefined()
	})
})

describe('WorkflowExecutionsFeature — a node looks the same as in the editor', () => {
	function withCondition(api: FakeApi) {
		api.mockGet(CONNECTORS_PATH, () => ({
			data: {
				auth_schemes: [],
				connectors: [
					{
						...descriptor('flow.loop', 'Pour chaque'),
						family: 'flow',
					},
					descriptor('mystery.thing', 'Chose inconnue'),
				],
			},
			pagination: null,
		}))
		api.mockGet(RUN_PATH, () => ({
			data: runDetail({
				graph: {
					triggers: [],
					connectors: [
						{ id: 'c1', kind: 'flow.loop', version: 1, config: {} },
						{ id: 'c2', kind: 'mystery.thing', version: 1, config: {} },
					],
					edges: [],
				},
				steps: [
					{
						id: 's1',
						connector_id: 'c1',
						iteration_path: '',
						attempts: 1,
						status: 'succeeded',
						created_at: '2026-08-05T00:00:01Z',
						started_at: '2026-08-05T00:00:01Z',
						output: { matched: true },
					},
				],
			}),
			pagination: null,
		}))
	}

	it('writes the subtitle of the kind under the node on the canvas', async () => {
		render(withCondition)

		const subtitles = await screen.findAllByText('Boucle')
		expect(subtitles.length).toBeGreaterThan(0)
		expect(screen.getByText('Connecteur')).toBeDefined()
	})

	it('draws the step of the Déroulé with the icon of its kind', async () => {
		render(withCondition)

		const steps = await screen.findByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		const button = within(steps).getByRole('button', { name: /Pour chaque/ })
		expect(button.querySelector('.lucide-repeat')).not.toBeNull()
	})

	it('draws the node of the canvas with the icon of its kind', async () => {
		render(withCondition)
		await screen.findByRole('complementary', { name: 'Déroulé de l’exécution' })

		const nodes = document.querySelectorAll('.react-flow__node')
		expect(nodes).toHaveLength(2)
		expect(nodes[0]?.querySelector('.lucide-repeat')).not.toBeNull()
		expect(nodes[1]?.querySelector('.lucide-repeat')).toBeNull()
	})
})

describe('WorkflowExecutionsFeature — a node is named as in the editor', () => {
	function withFailedCondition(api: FakeApi) {
		api.mockGet(CONNECTORS_PATH, () => ({
			data: {
				auth_schemes: [],
				connectors: [
					{
						...descriptor('flow.condition', 'Condition du serveur'),
						family: 'flow',
						branches: ['Then', 'Else'],
					},
				],
			},
			pagination: null,
		}))
		api.mockGet(RUNS_PATH, () => ({
			data: [run({ status: 'failed' })],
			pagination: null,
		}))
		api.mockGet(RUN_PATH, () => ({
			data: runDetail({
				status: 'failed',
				graph: {
					triggers: [],
					connectors: [
						{ id: 'c1', kind: 'flow.condition', version: 1, config: {} },
					],
					edges: [],
				},
				steps: [
					{
						id: 's1',
						connector_id: 'c1',
						iteration_path: '',
						attempts: 1,
						status: 'dead',
						created_at: '2026-08-05T00:00:01Z',
						started_at: '2026-08-05T00:00:01Z',
						error: 'SMTP expiré',
					},
				],
			}),
			pagination: null,
		}))
	}

	it('names it in the failure banner, the Déroulé and the canvas', async () => {
		render(withFailedCondition)

		const alert = await screen.findByRole('alert')
		expect(alert.textContent).toContain('Échec sur « Si »')
		const steps = screen.getByRole('complementary', {
			name: 'Déroulé de l’exécution',
		})
		expect(within(steps).getByRole('button', { name: /Si/ })).toBeDefined()
		expect(screen.getAllByText('Si').length).toBeGreaterThan(1)
		expect(screen.queryByText('Condition du serveur')).toBeNull()
	})

	it('names it in the title and the type of the node details', async () => {
		render(withFailedCondition)
		await screen.findByRole('alert')

		await openStep('Si')

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByRole('heading', { name: 'Si' })).toBeDefined()
		expect(
			within(dialog).getByText(/^Si · Condition · Exécution #/),
		).toBeDefined()
	})
})
