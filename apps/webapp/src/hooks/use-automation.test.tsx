import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	useAutomationFolders,
	useAutomationWorkflows,
	useRunGraphs,
	useRunPolling,
	useSaveWorkflowVersion,
	useUpdateWorkflow,
	useWorkflow,
} from '#/hooks/use-automation'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'

function installFakeApi(statusAtCall: (call: number) => string) {
	let calls = 0

	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown }
			const queryKey = [{ _id: path, path: p.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls += 1
						return {
							data: { id: 'run-1', status: statusAtCall(calls), steps: [] },
							pagination: null,
						}
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

	return { getCalls: () => calls }
}

function renderPollingHook() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}
	return renderHook(() => useRunPolling('org-1', 'run-1'), { wrapper })
}

describe('useRunPolling', () => {
	beforeEach(() => {
		vi.useFakeTimers()
	})

	afterEach(() => {
		vi.useRealTimers()
	})

	it('keeps polling GET /runs/{id} every two seconds while the run is not terminal', async () => {
		const { getCalls } = installFakeApi(() => 'running')
		renderPollingHook()

		await vi.advanceTimersByTimeAsync(0)
		expect(getCalls()).toBe(1)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(2)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(3)
	})

	it('stops polling once the run reaches a terminal status', async () => {
		const { getCalls } = installFakeApi((call) =>
			call === 1 ? 'running' : 'succeeded',
		)
		renderPollingHook()

		await vi.advanceTimersByTimeAsync(0)
		expect(getCalls()).toBe(1)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(2)

		await vi.advanceTimersByTimeAsync(10_000)
		expect(getCalls()).toBe(2)
	})

	it('never fetches when there is no run to follow', async () => {
		const { getCalls } = installFakeApi(() => 'running')
		const queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		})
		function wrapper({ children }: { children: ReactNode }) {
			return (
				<QueryClientProvider client={queryClient}>
					{children}
				</QueryClientProvider>
			)
		}
		renderHook(() => useRunPolling('org-1', null), { wrapper })

		await vi.advanceTimersByTimeAsync(5000)
		expect(getCalls()).toBe(0)
	})
})

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const FOLDERS_PATH =
	'/api/v1/organizations/{organization_id}/automation/folders'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'

function stableWrapper() {
	const client = new QueryClient({
		defaultOptions: { queries: { retry: false, staleTime: Infinity } },
	})
	return ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>{children}</QueryClientProvider>
	)
}

function installWorkflowApi() {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, () => ({ data: [], pagination: null }))
	api.mockGet(WORKFLOW_PATH, () => ({
		data: { id: 'wf-1', tags: [] },
		pagination: null,
	}))
	api.mockGet(FOLDERS_PATH, () => ({ data: [], pagination: null }))
	api.mockMutation('patch', WORKFLOW_PATH, () => ({ data: {} }))
	api.mockMutation('put', VERSIONS_PATH, () => ({ data: {} }))
	const fetched = (path: string) =>
		api.calls.filter((call) => call.method === 'get' && call.path === path)
			.length
	return { api, fetched }
}

describe('workflow mutations — refreshing what the screens show', () => {
	it('refetches the list and the detail of a workflow once it is updated', async () => {
		const { fetched } = installWorkflowApi()
		const { result } = renderHook(
			() => ({
				list: useAutomationWorkflows('org-1'),
				detail: useWorkflow('org-1', 'wf-1'),
				update: useUpdateWorkflow(),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => {
			expect(result.current.list.isSuccess).toBe(true)
			expect(result.current.detail.isSuccess).toBe(true)
		})

		await act(() =>
			result.current.update.mutateAsync({
				path: { organization_id: 'org-1', workflow_id: 'wf-1' },
				body: { tags: ['Ventes'] },
			}),
		)

		await waitFor(() => {
			expect(fetched(WORKFLOWS_PATH)).toBe(2)
			expect(fetched(WORKFLOW_PATH)).toBe(2)
		})
	})

	it('refetches the list and the detail of a workflow once a version is saved', async () => {
		const { fetched } = installWorkflowApi()
		const { result } = renderHook(
			() => ({
				list: useAutomationWorkflows('org-1'),
				detail: useWorkflow('org-1', 'wf-1'),
				save: useSaveWorkflowVersion(),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => {
			expect(result.current.list.isSuccess).toBe(true)
			expect(result.current.detail.isSuccess).toBe(true)
		})

		await act(() =>
			result.current.save.mutateAsync({
				path: { organization_id: 'org-1', workflow_id: 'wf-1' },
				body: { graph: { connectors: [], edges: [], triggers: [] } },
			}),
		)

		await waitFor(() => {
			expect(fetched(WORKFLOWS_PATH)).toBe(2)
			expect(fetched(WORKFLOW_PATH)).toBe(2)
		})
	})
})

describe('workflow queries', () => {
	it('asks the list for the archived workflows only when told to', async () => {
		const { api } = installWorkflowApi()
		const { result } = renderHook(
			() => ({
				live: useAutomationWorkflows('org-1'),
				archived: useAutomationWorkflows('org-1', true),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => {
			expect(result.current.live.isSuccess).toBe(true)
			expect(result.current.archived.isSuccess).toBe(true)
		})

		const queries = api.calls
			.filter((call) => call.path === WORKFLOWS_PATH)
			.map((call) => (call.params as { query?: unknown }).query)
		expect(queries).toContainEqual({ archived: true })
		expect(queries).toContainEqual({})
	})

	it('never fetches the detail of a workflow that has no id yet', async () => {
		const { fetched } = installWorkflowApi()
		const { result } = renderHook(() => useWorkflow('org-1', null), {
			wrapper: stableWrapper(),
		})

		await act(async () => {
			await Promise.resolve()
		})

		expect(result.current.fetchStatus).toBe('idle')
		expect(fetched(WORKFLOW_PATH)).toBe(0)
	})

	it('lists the folders of the organization', async () => {
		const { api } = installWorkflowApi()
		api.mockGet(FOLDERS_PATH, () => ({
			data: [
				{
					id: 'folder-1',
					name: 'Ventes',
					parent_id: null,
					created_at: '2026-11-01T00:00:00Z',
				},
			],
			pagination: null,
		}))
		const { result } = renderHook(() => useAutomationFolders('org-1'), {
			wrapper: stableWrapper(),
		})

		await waitFor(() => expect(result.current.isSuccess).toBe(true))

		expect(result.current.data?.data.map((folder) => folder.name)).toEqual([
			'Ventes',
		])
		expect(api.calls[0]?.params).toMatchObject({
			path: { organization_id: 'org-1' },
		})
	})
})

describe('useRunGraphs', () => {
	const GRAPH = {
		connectors: [{ id: 'c1', kind: 'mestier.mail', version: 1, config: {} }],
		edges: [],
		triggers: [],
	}

	function runDetail(runId: string, graph: unknown = GRAPH) {
		return {
			data: {
				id: runId,
				workflow_version_id: `version-of-${runId}`,
				graph,
				steps: [],
			},
			pagination: null,
		}
	}

	it('hands back the graph of each run under the version the run pins', async () => {
		const { api } = installWorkflowApi()
		api.mockGet(RUN_PATH, (params) =>
			runDetail((params as { path: { run_id: string } }).path.run_id),
		)
		const { result } = renderHook(
			() => useRunGraphs('org-1', ['run-a', 'run-b']),
			{ wrapper: stableWrapper() },
		)

		await waitFor(() => expect(result.current.size).toBe(2))

		expect(result.current.get('version-of-run-a')).toEqual(GRAPH)
		expect(result.current.get('version-of-run-b')).toEqual(GRAPH)
	})

	it('keeps a version whose graph is gone as an empty answer', async () => {
		const { api } = installWorkflowApi()
		api.mockGet(RUN_PATH, (params) =>
			runDetail((params as { path: { run_id: string } }).path.run_id, null),
		)
		const { result } = renderHook(() => useRunGraphs('org-1', ['run-a']), {
			wrapper: stableWrapper(),
		})

		await waitFor(() => expect(result.current.size).toBe(1))

		expect(result.current.get('version-of-run-a')).toBeNull()
	})

	it('says nothing of a run it could not load', async () => {
		const { api } = installWorkflowApi()
		api.mockGet(RUN_PATH, (params) => {
			const id = (params as { path: { run_id: string } }).path.run_id
			if (id === 'run-a') throw new Error('boom')
			return runDetail(id)
		})
		const { result } = renderHook(
			() => useRunGraphs('org-1', ['run-a', 'run-b']),
			{ wrapper: stableWrapper() },
		)

		await waitFor(() => expect(result.current.size).toBe(1))

		expect([...result.current.keys()]).toEqual(['version-of-run-b'])
	})

	it('fetches nothing when there is no run to ask about', async () => {
		const { fetched } = installWorkflowApi()
		const { result } = renderHook(() => useRunGraphs('org-1', []), {
			wrapper: stableWrapper(),
		})

		await act(async () => {
			await Promise.resolve()
		})

		expect(result.current.size).toBe(0)
		expect(fetched(RUN_PATH)).toBe(0)
	})

	it('keeps the same answer from one render to the next', async () => {
		const { api } = installWorkflowApi()
		api.mockGet(RUN_PATH, (params) =>
			runDetail((params as { path: { run_id: string } }).path.run_id),
		)
		const { result, rerender } = renderHook(
			() => useRunGraphs('org-1', ['run-a']),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => expect(result.current.size).toBe(1))
		const first = result.current

		rerender()

		expect(result.current).toBe(first)
	})
})
