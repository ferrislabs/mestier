import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { ExecutionsLogFeature } from '#/pages/automation/feature/executions-log-feature'
import { formatResumeNote } from '#/pages/automation/lib/format'
import {
	type FakeApi,
	installFakeTanstackApi,
} from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'

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

function run(overrides: Record<string, unknown>) {
	return {
		id: 'run-1',
		number: 1,
		organization_id: 'org-1',
		workflow_id: 'wf-1',
		workflow_version_id: 'v1v1v1v1',
		workflow_version_created_at: new Date(2026, 9, 31, 18, 20).toISOString(),
		status: 'succeeded',
		steps_done: 3,
		steps_total: 3,
		created_at: '2026-08-01T00:00:00Z',
		started_at: '2026-08-01T00:00:00Z',
		finished_at: '2026-08-01T00:00:03Z',
		...overrides,
	}
}

function workflow(id: string, name: string) {
	return {
		id,
		organization_id: 'org-1',
		name,
		enabled: true,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
	}
}

const DEFAULT_RUNS = [
	run({ id: 'aaaaaaaa-1', number: 1285 }),
	run({
		id: 'bbbbbbbb-2',
		number: 1286,
		status: 'failed',
		error: 'SMTP expiré',
		steps_done: 1,
		steps_total: 3,
	}),
	run({ id: 'cccccccc-3', number: 1287, workflow_id: 'gone' }),
]

function render(
	onFiltersChange = vi.fn(),
	{
		runs = DEFAULT_RUNS,
		workflows = [workflow('wf-1', 'Relance des devis')],
		archived = [],
		configure = () => {},
	}: {
		runs?: Record<string, unknown>[]
		workflows?: ReturnType<typeof workflow>[]
		archived?: ReturnType<typeof workflow>[]
		configure?: (api: FakeApi) => void
	} = {},
) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, (params) => ({
		data: (params as { query?: { archived?: boolean } }).query?.archived
			? archived
			: workflows,
	}))
	api.mockGet(RUNS_PATH, () => ({ data: runs }))
	api.mockGet(EVENTS_PATH, () => ({ data: [] }))
	api.mockGet(CONNECTORS_PATH, () => ({
		data: { auth_schemes: [], connectors: [] },
	}))
	configure(api)

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider
				client={
					new QueryClient({ defaultOptions: { queries: { retry: false } } })
				}
			>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return {
		api,
		onFiltersChange,
		rendered: renderWithRouter(
			<Providers>
				<ExecutionsLogFeature
					filters={{
						status: 'all',
						workflow: 'all',
						trigger: 'all',
						period: 'all',
					}}
					onFiltersChange={onFiltersChange}
				/>
			</Providers>,
		),
	}
}

afterEach(() => {
	vi.useRealTimers()
})

describe('ExecutionsLogFeature', () => {
	it('lists every run of the organization under its workflow name', async () => {
		render()

		expect(await screen.findAllByText('Relance des devis')).toHaveLength(2)
		expect(screen.getByText('Workflow supprimé')).toBeDefined()
		expect(screen.getByText('SMTP expiré')).toBeDefined()
	})

	it('narrows the list as the search is typed', async () => {
		const user = userEvent.setup()
		render()
		await screen.findByText('SMTP expiré')

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher une exécution' }),
			'smtp',
		)

		expect(screen.queryByText('Workflow supprimé')).toBeNull()
		expect(screen.getByText('SMTP expiré')).toBeDefined()
	})

	it('opens a run on the executions view of its workflow, the run selected', async () => {
		const user = userEvent.setup()
		const { rendered } = render()
		await screen.findByText('SMTP expiré')

		await user.click(
			screen.getByRole('link', {
				name: 'Ouvrir l’exécution 1286 de Relance des devis',
			}),
		)

		const { router } = await rendered
		await waitFor(() =>
			expect(router.state.location.href).toBe(
				'/o/atelier-bois/automatisation/workflows/wf-1?vue=executions&execution=1286',
			),
		)
	})

	it('hands the status filter to the route', async () => {
		const user = userEvent.setup()
		const { onFiltersChange } = render()
		await screen.findByText('SMTP expiré')

		await user.click(screen.getByRole('radio', { name: /Échecs/ }))

		expect(onFiltersChange).toHaveBeenCalledWith({ status: 'error' })
	})

	it('offers only the workflows that have at least one run in the workflow filter', async () => {
		const user = userEvent.setup()
		render(vi.fn(), {
			workflows: [
				workflow('wf-1', 'Relance des devis'),
				workflow('wf-2', 'Sans aucune exécution'),
			],
		})
		await screen.findByText('SMTP expiré')

		await user.click(
			screen.getByRole('combobox', { name: 'Filtrer par workflow' }),
		)

		const listbox = await screen.findByRole('listbox')
		expect(
			within(listbox)
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual(['Tous les workflows', 'Relance des devis', 'Workflow supprimé'])
	})

	it('lists the runs that started at the same second by number, latest first', async () => {
		render(vi.fn(), {
			runs: [
				run({ id: '018f-a', number: 41 }),
				run({ id: '018f-c', number: 43 }),
				run({ id: '018f-b', number: 42 }),
			],
		})
		await screen.findAllByText('Relance des devis')

		expect(
			screen
				.getAllByRole('link', { name: /Ouvrir l’exécution/ })
				.map((link) => link.textContent),
		).toEqual(['#43', '#42', '#41'])
	})

	it('keeps counting the duration of a running run on its own, between two fetches', async () => {
		const start = new Date('2026-08-05T10:00:10.000Z').getTime()
		vi.useFakeTimers({ toFake: ['Date'], now: start })
		render(vi.fn(), {
			runs: [
				run({
					id: 'running-1',
					status: 'running',
					started_at: new Date(start - 2200).toISOString(),
					finished_at: null,
				}),
			],
		})

		expect(await screen.findByText('2,2 s')).toBeDefined()

		vi.setSystemTime(start + 1000)

		expect(await screen.findByText('3,2 s')).toBeDefined()
	})

	it('leaves the clock alone when every run is over', async () => {
		const start = new Date('2026-08-05T10:00:10.000Z').getTime()
		vi.useFakeTimers({ toFake: ['Date'], now: start })
		render()
		await screen.findByText('SMTP expiré')

		vi.setSystemTime(start + 5000)
		await new Promise((resolve) => setTimeout(resolve, 450))

		expect(screen.getAllByText('3 s').length).toBeGreaterThan(0)
	})

	it('writes the node counts and the date of the version of each run', async () => {
		render()
		await screen.findByText('SMTP expiré')

		expect(screen.getAllByText('1 / 3')).toHaveLength(1)
		expect(screen.getAllByText('3 / 3')).toHaveLength(2)
		expect(
			screen.getAllByText('Version du 31 oct. 2026 à 18:20').length,
		).toBeGreaterThan(0)
	})

	it('names a run after the event it carries and offers those names in the trigger filter', async () => {
		const user = userEvent.setup()
		render(vi.fn(), {
			runs: [
				run({
					id: 'evt-run',
					number: 5,
					trigger_event_id: 'evt-1',
					trigger_event_name: 'quote.accepted',
				}),
				run({
					id: 'evt-run-2',
					number: 6,
					trigger_event_id: 'evt-2',
					trigger_event_name: 'invoice.issued',
				}),
				run({ id: 'manual-run', number: 4 }),
			],
			configure: (api) =>
				api.mockGet(EVENTS_PATH, () => ({
					data: [
						{ name: 'quote.accepted', label: 'Devis accepté' },
						{ name: 'invoice.issued', label: 'Facture émise' },
					],
				})),
		})
		expect(await screen.findByText('Devis accepté')).toBeDefined()
		expect(screen.getByText('Facture émise')).toBeDefined()

		await user.click(
			screen.getByRole('combobox', { name: 'Filtrer par déclencheur' }),
		)

		const listbox = await screen.findByRole('listbox')
		expect(
			within(listbox)
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual([
			'Tous les déclencheurs',
			'Devis accepté',
			'Facture émise',
			'Test manuel',
		])
	})

	describe('the webhook and the scheduler', () => {
		const GRAPH = {
			triggers: [
				{
					id: 't1',
					kind: { Webhook: { method: 'POST', path: 'paiement-recu' } },
				},
				{
					id: 't2',
					kind: { Schedule: { frequency: 'Daily', time: '09:00' } },
				},
				{
					id: 't3',
					name: 'Paiement Stripe',
					kind: { Webhook: { method: 'POST', path: 'stripe' } },
				},
			],
			connectors: [],
			edges: [],
		}
		const RUNS = [
			run({ id: 'hook-run', number: 9, trigger_id: 't1' }),
			run({ id: 'cron-run', number: 8, trigger_id: 't2' }),
			run({ id: 'stripe-run', number: 7, trigger_id: 't3' }),
			run({ id: 'manual-run', number: 6 }),
		]

		function configureGraph(api: FakeApi) {
			api.mockGet(RUN_PATH, () => ({
				data: {
					...run({ workflow_version_id: 'v1v1v1v1' }),
					graph: GRAPH,
					steps: [],
				},
			}))
		}

		it('names a run after the trigger node that started it, with the default names of the two types', async () => {
			render(vi.fn(), { runs: RUNS, configure: configureGraph })

			expect(await screen.findByText('Webhook')).toBeDefined()
			expect(screen.getByText('Planificateur')).toBeDefined()
			expect(screen.getByText('Paiement Stripe')).toBeDefined()
			expect(screen.getByText('Test manuel')).toBeDefined()
			expect(screen.queryByRole('cell', { name: 'Déclencheur' })).toBeNull()
		})

		it('asks for the graph of a version once, and not for the runs a label does not need it for', async () => {
			const { api } = render(vi.fn(), { runs: RUNS, configure: configureGraph })
			await screen.findByText('Webhook')

			expect(api.calls.filter((call) => call.path === RUN_PATH)).toHaveLength(1)
		})

		it('derives the trigger filter from these names', async () => {
			const user = userEvent.setup()
			render(vi.fn(), { runs: RUNS, configure: configureGraph })
			await screen.findByText('Webhook')

			await user.click(
				screen.getByRole('combobox', { name: 'Filtrer par déclencheur' }),
			)

			const listbox = await screen.findByRole('listbox')
			expect(
				within(listbox)
					.getAllByRole('option')
					.map((option) => option.textContent),
			).toEqual([
				'Tous les déclencheurs',
				'Paiement Stripe',
				'Planificateur',
				'Test manuel',
				'Webhook',
			])
		})

		it('says « Déclencheur » until the graph of the version is known', async () => {
			render(vi.fn(), {
				runs: [run({ id: 'hook-run', number: 9, trigger_id: 't1' })],
				configure: (api) => api.mockGet(RUN_PATH, () => new Promise(() => {})),
			})

			expect(
				await screen.findByRole('cell', { name: 'Déclencheur' }),
			).toBeDefined()
		})
	})

	describe('the node that failed', () => {
		const GRAPH = {
			triggers: [],
			connectors: [
				{ id: 'c1', kind: 'mestier.mail', version: 1, config: {} },
				{
					id: 'c2',
					kind: 'mestier.mail',
					version: 1,
					config: {},
					name: 'Envoyer l’e-mail',
				},
			],
			edges: [],
		}

		function failedRun(id: string, number: number, version = 'v1v1v1v1') {
			return run({
				id,
				number,
				status: 'failed',
				error: 'SMTP expiré',
				failed_connector_id: 'c2',
				workflow_version_id: version,
			})
		}

		function withGraph(api: FakeApi) {
			api.mockGet(RUN_PATH, (params) => ({
				data: {
					...failedRun((params as { path: { run_id: string } }).path.run_id, 1),
					graph: GRAPH,
					steps: [],
				},
			}))
		}

		it('is named after the graph of the run, in red, before the message', async () => {
			render(vi.fn(), {
				runs: [failedRun('failed-1', 10)],
				configure: withGraph,
			})

			const name = await screen.findByText('Envoyer l’e-mail')
			expect(name.className).toContain('text-state-alert')
			expect(screen.getByText(': SMTP expiré')).toBeDefined()
		})

		it('asks the graph once per version, whatever the number of failed runs', async () => {
			const { api, rendered } = render(vi.fn(), {
				runs: [
					failedRun('failed-1', 10),
					failedRun('failed-2', 11),
					failedRun('failed-3', 12, 'v2v2v2v2'),
				],
				configure: withGraph,
			})
			await rendered
			await screen.findAllByText('Envoyer l’e-mail')

			expect(api.calls.filter((call) => call.path === RUN_PATH)).toHaveLength(2)
		})

		it('is found by the search', async () => {
			const user = userEvent.setup()
			render(vi.fn(), {
				runs: [failedRun('failed-1', 10), run({ id: 'fine', number: 11 })],
				configure: withGraph,
			})
			await screen.findByText('Envoyer l’e-mail')

			await user.type(
				screen.getByRole('searchbox', { name: 'Rechercher une exécution' }),
				'envoyer l’e-mail',
			)

			expect(screen.queryByRole('link', { name: /exécution 11 /i })).toBeNull()
			expect(screen.getByRole('link', { name: /exécution 10 /i })).toBeDefined()
		})

		it('falls back to its identifier when the version of the run is gone', async () => {
			render(vi.fn(), {
				runs: [failedRun('failed-1', 10)],
				configure: (api) =>
					api.mockGet(RUN_PATH, () => ({
						data: { ...failedRun('failed-1', 10), graph: null, steps: [] },
					})),
			})

			expect(await screen.findByText('c2')).toBeDefined()
			expect(screen.getByText(': SMTP expiré')).toBeDefined()
		})

		it('leaves the message alone when the graph cannot be loaded', async () => {
			render(vi.fn(), {
				runs: [failedRun('failed-1', 10)],
				configure: (api) =>
					api.mockGet(RUN_PATH, () => {
						throw new Error('boom')
					}),
			})

			const message = await screen.findByText('SMTP expiré')
			expect(message.className).toContain('text-state-alert')
		})
	})

	it('names the runs of an archived workflow instead of calling it deleted', async () => {
		render(vi.fn(), {
			runs: [run({ id: 'archived-run', number: 8, workflow_id: 'wf-old' })],
			workflows: [],
			archived: [workflow('wf-old', 'Ancienne relance')],
		})

		expect(await screen.findByText('Ancienne relance')).toBeDefined()
		expect(screen.queryByText('Workflow supprimé')).toBeNull()
	})

	it('shows a run that waits as paused, with no duration and the time it resumes', async () => {
		const resume = new Date(Date.now() + 2 * 86_400_000).toISOString()
		render(vi.fn(), {
			runs: [
				run({
					id: 'parked-1',
					number: 31,
					status: 'running',
					started_at: '2026-08-01T00:00:00Z',
					finished_at: null,
					next_attempt_at: resume,
				}),
			],
		})

		const row = (
			await screen.findByRole('link', { name: /exécution 31/ })
		).closest('tr') as HTMLElement
		expect(within(row).getByText('En pause')).toBeDefined()
		expect(within(row).queryByText('En cours')).toBeNull()
		expect(within(row).getByText(formatResumeNote(resume) ?? '')).toBeDefined()
		expect(within(row).getByText('—')).toBeDefined()
	})

	it('still counts a run that waits among the runs in progress', async () => {
		const resume = new Date(Date.now() + 2 * 86_400_000).toISOString()
		render(vi.fn(), {
			runs: [
				run({
					id: 'parked-1',
					number: 31,
					status: 'running',
					finished_at: null,
					next_attempt_at: resume,
				}),
				run({ id: 'done-1', number: 30 }),
			],
		})

		await screen.findByRole('link', { name: /exécution 31/ })

		expect(
			screen.getByRole('radio', { name: /En cours/ }).textContent,
		).toContain('1')
	})
})
