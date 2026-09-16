import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkflowRunsFeature } from '#/pages/automation/feature/workflow-runs-feature'

const { navigateSpy } = vi.hoisted(() => ({ navigateSpy: vi.fn() }))

vi.mock('@tanstack/react-router', async (importOriginal) => {
	const actual = await importOriginal<typeof import('@tanstack/react-router')>()
	return { ...actual, useNavigate: () => navigateSpy }
})

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'

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

function workflowDetail() {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Créer une facture Odoo',
		description: null,
		enabled: true,
		layout: null,
		current_version: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
	}
}

type Handler = (params: unknown) => unknown

function installFakeTanstackApi() {
	const getHandlers = new Map<string, Handler>()

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
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
					},
				},
			}
		},
		mutation() {
			throw new Error('unused in this test')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		mockGet: (path: string, handler: Handler) => getHandlers.set(path, handler),
	}
}

function renderFeature(
	configure: (api: ReturnType<typeof installFakeTanstackApi>) => void,
) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOW_PATH, () => ({
		data: workflowDetail(),
		pagination: null,
	}))
	api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
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
			<WorkflowRunsFeature workflowId="workflow-1" />
		</Providers>,
	)
}

beforeEach(() => {
	navigateSpy.mockReset()
})

describe('WorkflowRunsFeature — listing', () => {
	it('lists only this workflow’s runs, most recent first', async () => {
		renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					{
						id: 'run-old',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-01T00:00:00Z',
						started_at: '2026-08-01T00:00:00.000Z',
						finished_at: '2026-08-01T00:00:01.000Z',
					},
					{
						id: 'run-other-workflow',
						organization_id: 'org-1',
						workflow_id: 'workflow-2',
						workflow_version_id: 'v1',
						status: 'failed',
						created_at: '2026-08-05T00:00:00Z',
					},
					{
						id: 'run-new',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'running',
						created_at: '2026-08-03T00:00:00Z',
					},
				],
				pagination: null,
			}))
		})

		const rows = await screen.findAllByRole('row')
		expect(rows).toHaveLength(3)
		const bodyRows = rows.slice(1)
		expect(bodyRows[0]?.textContent).toMatch(/En cours/)
		expect(bodyRows[1]?.textContent).toMatch(/Réussi/)
	})
})

describe('WorkflowRunsFeature — navigation', () => {
	it('opens a run’s inspector', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					{
						id: 'run-1',
						organization_id: 'org-1',
						workflow_id: 'workflow-1',
						workflow_version_id: 'v1',
						status: 'succeeded',
						created_at: '2026-08-01T00:00:00Z',
					},
				],
				pagination: null,
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Détails' }))

		expect(navigateSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				params: { workflowId: 'workflow-1', runId: 'run-1' },
			}),
		)
	})

	it('goes back to the editor', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: /Retour à l’éditeur/ }),
		)

		expect(navigateSpy).toHaveBeenCalledWith(
			expect.objectContaining({ params: { workflowId: 'workflow-1' } }),
		)
	})
})
