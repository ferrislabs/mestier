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
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'

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
		organization_id: 'org-1',
		workflow_id: 'wf-1',
		workflow_version_id: 'v1v1v1v1',
		status: 'succeeded',
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
	run({ id: 'aaaaaaaa-1' }),
	run({ id: 'bbbbbbbb-2', status: 'failed', error: 'SMTP expiré' }),
	run({ id: 'cccccccc-3', workflow_id: 'gone' }),
]

function render(
	onFiltersChange = vi.fn(),
	{
		runs = DEFAULT_RUNS,
		workflows = [workflow('wf-1', 'Relance des devis')],
	}: {
		runs?: Record<string, unknown>[]
		workflows?: ReturnType<typeof workflow>[]
	} = {},
) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, () => ({ data: workflows }))
	api.mockGet(RUNS_PATH, () => ({ data: runs }))

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
				name: 'Ouvrir l’exécution bbbbbbbb de Relance des devis',
			}),
		)

		const { router } = await rendered
		await waitFor(() =>
			expect(router.state.location.href).toBe(
				'/o/atelier-bois/automatisation/workflows/wf-1?vue=executions&execution=bbbbbbbb-2',
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

	it('lists the runs that started at the same second latest first', async () => {
		render(vi.fn(), {
			runs: [
				run({ id: '018f-a' }),
				run({ id: '018f-c' }),
				run({ id: '018f-b' }),
			],
		})
		await screen.findAllByText('Relance des devis')

		expect(
			screen
				.getAllByRole('link', { name: /Ouvrir l’exécution/ })
				.map((link) => link.textContent),
		).toEqual(['#018f-c', '#018f-b', '#018f-a'])
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
})
