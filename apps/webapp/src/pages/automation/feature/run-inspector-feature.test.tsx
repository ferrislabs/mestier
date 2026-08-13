import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { RunInspectorFeature } from '#/pages/automation/feature/run-inspector-feature'
import { renderWithRouter } from '#/test/render-with-router'

const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const RUN_REPLAY_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const RUN_DETAIL = {
	id: 'run-1',
	organization_id: 'org-1',
	workflow_id: 'workflow-1',
	workflow_version_id: 'version-1',
	status: 'failed',
	error: 'send-email failed',
	created_at: '2026-08-01T10:00:00Z',
	started_at: '2026-08-01T10:00:00Z',
	finished_at: '2026-08-01T10:00:05Z',
	steps: [
		{
			id: 'step-1',
			connector_id: 'fetch-quote',
			status: 'succeeded',
			attempts: 1,
			iteration_path: '',
			input: { quote_id: 'quote-1' },
			output: { amount: 1200 },
		},
		{
			id: 'step-2',
			connector_id: 'send-reminder',
			status: 'succeeded',
			attempts: 1,
			iteration_path: 'loop1[0]',
			input: { contact: 'a@example.com' },
			output: null,
		},
		{
			id: 'step-3',
			connector_id: 'send-email',
			status: 'failed',
			attempts: 2,
			error: 'timeout',
			iteration_path: '',
			input: null,
			output: null,
		},
	],
}

interface FakeApiHandlers {
	replay?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
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
						if (path === RUN_PATH) {
							return { data: RUN_DETAIL }
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
						if (method === 'post' && path === RUN_REPLAY_PATH) {
							if (!handlers.replay) throw new Error('replay not mocked')
							return handlers.replay(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake
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
			<RunInspectorFeature workflowId="workflow-1" runId="run-1" />
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

describe('RunInspectorFeature — steps and grouping', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('groups the loop step under an iteration header, distinct from top-level steps', async () => {
		await renderFeature()

		expect(await screen.findByText('fetch-quote')).toBeDefined()
		expect(screen.getByText('Itération 0')).toBeDefined()
		expect(screen.getByText('send-reminder')).toBeDefined()
		expect(screen.getByText('send-email')).toBeDefined()
	})
})

describe('RunInspectorFeature — replay from a step', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("replays from the failed step's connector once the run allows it (failed => canReplay)", async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			replay: () => ({ data: { run_id: 'run-2' } }),
		})

		await screen.findByText('send-email')
		const replayButtons = screen.getAllByRole('button', {
			name: /Relancer depuis ici/,
		})
		// One replay button per step since the run's status (failed) allows it.
		expect(replayButtons.length).toBe(3)
		await user.click(replayButtons[2])

		await waitFor(() => {
			const replayCall = calls.find(
				(c) => c.method === 'post' && c.path === RUN_REPLAY_PATH,
			)
			expect(replayCall?.params).toMatchObject({
				path: { organization_id: 'org-1', run_id: 'run-1' },
				body: { connector_id: 'send-email' },
			})
		})
	})
})
