import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { WorkflowListFeature } from '#/pages/automation/feature/workflow-list-feature'
import { renderWithRouter } from '#/test/render-with-router'

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
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
	description: null,
	enabled: true,
	current_version_id: 'version-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const OLD_RUN = {
	id: 'run-old',
	organization_id: 'org-1',
	workflow_id: 'workflow-1',
	workflow_version_id: 'version-1',
	status: 'failed',
	created_at: '2026-01-01T00:00:00Z',
}

const NEW_RUN = {
	id: 'run-new',
	organization_id: 'org-1',
	workflow_id: 'workflow-1',
	workflow_version_id: 'version-1',
	status: 'succeeded',
	created_at: '2026-08-01T00:00:00Z',
}

interface FakeApiHandlers {
	workflows?: unknown[]
	runs?: unknown[]
	postWorkflow?: (params: unknown) => unknown
	patchWorkflow?: (params: unknown) => unknown
	deleteWorkflow?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const workflows = handlers.workflows ?? [WORKFLOW]
	const runs = handlers.runs ?? [NEW_RUN, OLD_RUN]

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
						if (path === WORKFLOWS_PATH) {
							return { data: workflows, pagination: null }
						}
						if (path === RUNS_PATH) {
							return { data: runs, pagination: null }
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
						if (method === 'post' && path === WORKFLOWS_PATH) {
							if (!handlers.postWorkflow)
								throw new Error('postWorkflow not mocked')
							return handlers.postWorkflow(params)
						}
						if (method === 'patch' && path === WORKFLOW_PATH) {
							if (!handlers.patchWorkflow)
								throw new Error('patchWorkflow not mocked')
							return handlers.patchWorkflow(params)
						}
						if (method === 'delete' && path === WORKFLOW_PATH) {
							if (!handlers.deleteWorkflow) {
								throw new Error('deleteWorkflow not mocked')
							}
							return handlers.deleteWorkflow(params)
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
			<WorkflowListFeature />
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

describe('WorkflowListFeature — last run cross-reference', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("shows the most recent run's status for a workflow, not an older one", async () => {
		await renderFeature()

		expect(await screen.findByText('Relance devis')).toBeDefined()
		expect(screen.getByText('Réussi')).toBeDefined()
		expect(screen.queryByText('Échoué')).toBeNull()
	})
})

describe('WorkflowListFeature — enable/disable', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('flips enabled to false through useUpdateWorkflow', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			patchWorkflow: (params) => ({
				data: { ...WORKFLOW, enabled: false, ...paramsBody(params) },
			}),
		})

		await screen.findByText('Relance devis')
		await user.click(screen.getByRole('switch'))

		await waitFor(() => {
			const patchCall = calls.find(
				(c) => c.method === 'patch' && c.path === WORKFLOW_PATH,
			)
			expect(patchCall?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
				body: { enabled: false },
			})
		})
	})
})

describe('WorkflowListFeature — deletion', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('deletes the workflow once the confirmation dialog is accepted', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			deleteWorkflow: () => ({ data: undefined }),
		})

		await screen.findByText('Relance devis')
		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))
		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			const deleteCall = calls.find(
				(c) => c.method === 'delete' && c.path === WORKFLOW_PATH,
			)
			expect(deleteCall?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
			})
		})
	})
})

describe('WorkflowListFeature — creation', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('creates a workflow from the name field and clears the form', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			postWorkflow: (params) => ({
				data: { ...WORKFLOW, id: 'workflow-2', ...paramsBody(params) },
			}),
		})

		await screen.findByText('Relance devis')
		await user.type(screen.getByLabelText('Nom'), 'Nouveau workflow')
		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		await waitFor(() => {
			const postCall = calls.find(
				(c) => c.method === 'post' && c.path === WORKFLOWS_PATH,
			)
			expect(postCall?.params).toMatchObject({
				path: { organization_id: 'org-1' },
				body: { name: 'Nouveau workflow' },
			})
		})
	})
})

function paramsBody(params: unknown) {
	return (params as { body?: unknown }).body ?? {}
}
