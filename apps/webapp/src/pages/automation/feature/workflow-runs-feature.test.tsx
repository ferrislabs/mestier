import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { WorkflowRunsFeature } from '#/pages/automation/feature/workflow-runs-feature'
import { renderWithRouter } from '#/test/render-with-router'

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const WORKFLOW = {
	id: 'workflow-1',
	organization_id: 'org-1',
	name: 'Relance devis',
	enabled: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const MATCHING_RUN = {
	id: 'run-1',
	organization_id: 'org-1',
	workflow_id: 'workflow-1',
	workflow_version_id: 'version-1',
	status: 'succeeded',
	created_at: '2026-08-01T00:00:00Z',
}

const OTHER_WORKFLOW_RUN = {
	id: 'run-2',
	organization_id: 'org-1',
	workflow_id: 'workflow-other',
	workflow_version_id: 'version-1',
	status: 'failed',
	created_at: '2026-08-02T00:00:00Z',
}

function installFakeTanstackApi() {
	const calls: { method: string; path: string; params: unknown }[] = []

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
						if (path === WORKFLOW_PATH) {
							return { data: WORKFLOW }
						}
						if (path === RUNS_PATH) {
							return {
								data: [OTHER_WORKFLOW_RUN, MATCHING_RUN],
								pagination: null,
							}
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			throw new Error('no mutation expected on the workflow runs page')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake
	;(window as any).tanstackApi = fakeApi

	return calls
}

function renderFeature() {
	const calls = installFakeTanstackApi()
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
			<WorkflowRunsFeature workflowId="workflow-1" />
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

describe('WorkflowRunsFeature — client-side filtering', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("filters the org-wide run list down to this workflow's runs only", async () => {
		await renderFeature()

		expect(await screen.findByText('Runs — Relance devis')).toBeDefined()
		expect(screen.getByText('Réussi')).toBeDefined()
		expect(screen.queryByText('Échoué')).toBeNull()
	})

	it('never calls a per-workflow runs endpoint — only the org-wide list', async () => {
		const { calls } = await renderFeature()
		await screen.findByText('Runs — Relance devis')

		const runsCalls = calls.filter(
			(c) => c.method === 'get' && c.path === RUNS_PATH,
		)
		expect(runsCalls.length).toBeGreaterThan(0)
		expect(
			calls.some((c) => c.path.includes('/workflows/{workflow_id}/runs')),
		).toBe(false)
	})
})
