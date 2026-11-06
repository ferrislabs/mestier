import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	useAutomationFolders,
	useAutomationWorkflows,
	useCollection,
	useCollectionDetails,
	useCollections,
	useDeleteCollection,
	useImportCollection,
	useReplaceCollection,
	useRunGraphs,
	useRunPolling,
	useSaveWorkflowVersion,
	useUpdateWorkflow,
	useWorkflow,
} from '#/hooks/use-automation'
import type { CollectionDraft } from '#/pages/automation/lib/collections'
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

const COLLECTIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections'
const COLLECTION_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections/{collection_id}'

const ADDRESS_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'
const COMPANY_ID = '0b7a3c1e-2d54-4c85-8b0d-6a4f1d2e9a77'

function storedCollection(
	id: string,
	overrides: Partial<Schemas.CollectionResponse> = {},
): Schemas.CollectionResponse {
	return {
		id,
		organization_id: 'org-1',
		slug: 'api-adresse',
		title: 'API Adresse',
		version: '1.0.0',
		description: 'Addresses',
		base_url: 'https://api.example.com/v1',
		source: { kind: 'url', url: 'https://api.example.com/openapi.json' },
		imported_at: '2026-11-06T09:30:00Z',
		security: [
			{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } },
		],
		operations: [
			{
				id: 'createAddress',
				method: 'POST',
				path: '/addresses',
				summary: 'Create',
				tag: 'Addresses',
				parameters: [],
				request_body: { schema: { type: 'object' } },
				response: null,
				response_status: '201',
			},
		],
		schemas: { Address: { type: 'object' } },
		history: [],
		...overrides,
	}
}

function summaryOf(
	collection: Schemas.CollectionResponse,
): Schemas.CollectionSummaryResponse {
	return {
		id: collection.id,
		organization_id: collection.organization_id,
		slug: collection.slug,
		title: collection.title,
		version: collection.version,
		base_url: collection.base_url,
		source: collection.source,
		imported_at: collection.imported_at,
		security: collection.security,
		operation_count: collection.operations.length,
	}
}

const DRAFT: CollectionDraft = {
	title: 'API Adresse',
	version: '1.1.0',
	description: 'Addresses',
	baseUrl: 'https://api.example.com/v1',
	source: { kind: 'file', fileName: 'adresse.json' },
	security: [
		{
			name: 'cle',
			kind: { type: 'api_key', location: 'header', parameterName: 'X-Key' },
		},
	],
	operations: [
		{
			id: 'createAddress',
			method: 'POST',
			path: '/addresses',
			summary: 'Create',
			tag: 'Addresses',
			parameters: [],
			requestBody: { schema: { type: 'object' } },
			response: null,
			responseStatus: '201',
		},
	],
	schemas: {},
}

function installCollectionsApi() {
	const api = installFakeTanstackApi()
	const address = storedCollection(ADDRESS_ID)
	const company = storedCollection(COMPANY_ID, {
		slug: 'recherche-entreprises',
		title: 'Recherche d’entreprises',
	})
	api.mockGet(COLLECTIONS_PATH, () => ({
		data: [summaryOf(address), summaryOf(company)],
		pagination: null,
	}))
	api.mockGet(COLLECTION_PATH, (params) => {
		const id = (params as { path: { collection_id: string } }).path
			.collection_id
		return { data: id === COMPANY_ID ? company : address, pagination: null }
	})
	api.mockMutation('post', COLLECTIONS_PATH, () => ({
		data: storedCollection(ADDRESS_ID),
	}))
	api.mockMutation('put', COLLECTION_PATH, () => ({
		data: storedCollection(ADDRESS_ID, { version: '1.1.0' }),
	}))
	api.mockMutation('delete', COLLECTION_PATH, () => undefined)
	const fetched = (path: string) =>
		api.calls.filter((call) => call.method === 'get' && call.path === path)
			.length
	return { api, fetched }
}

describe('collection queries', () => {
	it('lists the collections of the organization as camelCase summaries', async () => {
		const { api } = installCollectionsApi()
		const { result } = renderHook(() => useCollections('org-1'), {
			wrapper: stableWrapper(),
		})

		await waitFor(() => expect(result.current.isSuccess).toBe(true))

		expect(api.calls[0]?.params).toMatchObject({
			path: { organization_id: 'org-1' },
		})
		expect(result.current.data?.map((item) => item.title)).toEqual([
			'API Adresse',
			'Recherche d’entreprises',
		])
		expect(result.current.data?.[0]).toMatchObject({
			id: ADDRESS_ID,
			baseUrl: 'https://api.example.com/v1',
			importedAt: '2026-11-06T09:30:00Z',
			operationCount: 1,
		})
		expect(result.current.data?.[0]).not.toHaveProperty('operations')
	})

	it('reads one collection with its operations, converted to the front model', async () => {
		const { api } = installCollectionsApi()
		const { result } = renderHook(() => useCollection('org-1', COMPANY_ID), {
			wrapper: stableWrapper(),
		})

		await waitFor(() => expect(result.current.isSuccess).toBe(true))

		expect(api.calls[0]?.params).toMatchObject({
			path: { organization_id: 'org-1', collection_id: COMPANY_ID },
		})
		expect(result.current.data?.slug).toBe('recherche-entreprises')
		expect(result.current.data?.operations[0]?.requestBody).toEqual({
			schema: { type: 'object' },
		})
		expect(result.current.data?.operations[0]?.responseStatus).toBe('201')
	})

	it('does not retry a collection that does not exist', async () => {
		vi.useFakeTimers()
		try {
			const { api } = installCollectionsApi()
			api.mockGet(COLLECTION_PATH, () => {
				throw Object.assign(new Error('Resource not found'), { status: 404 })
			})
			const client = new QueryClient()
			const { result } = renderHook(() => useCollection('org-1', 'nope'), {
				wrapper: ({ children }: { children: ReactNode }) => (
					<QueryClientProvider client={client}>{children}</QueryClientProvider>
				),
			})

			await vi.advanceTimersByTimeAsync(10_000)

			expect(result.current.isError).toBe(true)
			expect(
				api.calls.filter((call) => call.path === COLLECTION_PATH),
			).toHaveLength(1)
		} finally {
			vi.useRealTimers()
		}
	})

	it('hands back every collection in full for the screens that need the operations', async () => {
		const { fetched } = installCollectionsApi()
		const { result } = renderHook(
			() => useCollectionDetails('org-1', [ADDRESS_ID, COMPANY_ID]),
			{ wrapper: stableWrapper() },
		)

		await waitFor(() => expect(result.current.collections.length).toBe(2))

		expect(result.current.collections.map((item) => item.id)).toEqual([
			ADDRESS_ID,
			COMPANY_ID,
		])
		expect(result.current.collections[0]?.operations).toHaveLength(1)
		expect(fetched(COLLECTION_PATH)).toBe(2)
	})

	it('does not read a collection while it is disabled, and reads it once enabled', async () => {
		const { fetched } = installCollectionsApi()
		const { result, rerender } = renderHook(
			({ enabled }: { enabled: boolean }) =>
				useCollection('org-1', ADDRESS_ID, { enabled }),
			{ wrapper: stableWrapper(), initialProps: { enabled: false } },
		)

		await act(async () => {
			await Promise.resolve()
		})
		expect(fetched(COLLECTION_PATH)).toBe(0)
		expect(result.current.fetchStatus).toBe('idle')

		rerender({ enabled: true })

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(fetched(COLLECTION_PATH)).toBe(1)
	})

	it('tells, for each collection asked for, whether it is loading, found, gone or unreadable', async () => {
		const { api } = installCollectionsApi()
		const gone = '11111111-1111-4111-8111-111111111111'
		const broken = '22222222-2222-4222-8222-222222222222'
		const pending = '33333333-3333-4333-8333-333333333333'
		api.mockGet(COLLECTION_PATH, (params) => {
			const id = (params as { path: { collection_id: string } }).path
				.collection_id
			if (id === gone) {
				throw Object.assign(new Error('Resource not found'), { status: 404 })
			}
			if (id === broken) {
				throw Object.assign(new Error('Internal server error'), {
					status: 500,
				})
			}
			if (id === pending) return new Promise(() => {})
			return { data: storedCollection(ADDRESS_ID), pagination: null }
		})
		const client = new QueryClient({
			defaultOptions: { queries: { retryDelay: 0 } },
		})
		const { result } = renderHook(
			() => useCollectionDetails('org-1', [ADDRESS_ID, gone, broken, pending]),
			{
				wrapper: ({ children }: { children: ReactNode }) => (
					<QueryClientProvider client={client}>{children}</QueryClientProvider>
				),
			},
		)

		await waitFor(() =>
			expect(result.current.reads.get(broken)?.status).toBe('error'),
		)

		expect(result.current.reads.get(ADDRESS_ID)?.status).toBe('found')
		expect(result.current.reads.get(gone)).toEqual({ status: 'notFound' })
		expect(result.current.reads.get(pending)).toEqual({ status: 'loading' })
		expect(result.current.collections.map((item) => item.id)).toEqual([
			ADDRESS_ID,
		])
		expect(result.current.isLoading).toBe(true)
	})

	it('does not wait on a collection that does not exist, but retries one it could not read', async () => {
		vi.useFakeTimers()
		try {
			const { api } = installCollectionsApi()
			const gone = '11111111-1111-4111-8111-111111111111'
			const broken = '22222222-2222-4222-8222-222222222222'
			api.mockGet(COLLECTION_PATH, (params) => {
				const id = (params as { path: { collection_id: string } }).path
					.collection_id
				throw Object.assign(new Error('failure'), {
					status: id === gone ? 404 : 500,
				})
			})
			const client = new QueryClient()
			const { result } = renderHook(
				() => useCollectionDetails('org-1', [gone, broken]),
				{
					wrapper: ({ children }: { children: ReactNode }) => (
						<QueryClientProvider client={client}>
							{children}
						</QueryClientProvider>
					),
				},
			)

			await vi.advanceTimersByTimeAsync(100)

			expect(result.current.reads.get(gone)).toEqual({ status: 'notFound' })
			expect(result.current.reads.get(broken)).toEqual({ status: 'loading' })
			const callsFor = (id: string) =>
				api.calls.filter(
					(call) =>
						call.path === COLLECTION_PATH &&
						(call.params as { path: { collection_id: string } }).path
							.collection_id === id,
				).length
			expect(callsFor(gone)).toBe(1)
			expect(callsFor(broken)).toBeGreaterThanOrEqual(1)

			await vi.advanceTimersByTimeAsync(10_000)

			expect(result.current.reads.get(broken)).toEqual({ status: 'error' })
			expect(callsFor(broken)).toBe(4)
			expect(callsFor(gone)).toBe(1)
		} finally {
			vi.useRealTimers()
		}
	})

	it('keeps the same reads between renders while nothing changes', async () => {
		installCollectionsApi()
		const { result, rerender } = renderHook(
			() => useCollectionDetails('org-1', [ADDRESS_ID]),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => expect(result.current.collections.length).toBe(1))
		const before = result.current.reads

		rerender()

		expect(result.current.reads).toBe(before)
	})

	it('fetches nothing when there is no collection to read', async () => {
		const { fetched } = installCollectionsApi()
		const { result } = renderHook(() => useCollectionDetails('org-1', []), {
			wrapper: stableWrapper(),
		})

		await act(async () => {
			await Promise.resolve()
		})

		expect(result.current.collections).toEqual([])
		expect(result.current.reads.size).toBe(0)
		expect(result.current.isLoading).toBe(false)
		expect(fetched(COLLECTION_PATH)).toBe(0)
	})
})

describe('collection mutations — refreshing what the screens show', () => {
	it('posts the draft in the API shape and hands back the stored collection', async () => {
		const { api, fetched } = installCollectionsApi()
		const { result } = renderHook(
			() => ({
				list: useCollections('org-1'),
				add: useImportCollection('org-1'),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		const stored = await act(() => result.current.add.mutateAsync(DRAFT))

		expect(stored.id).toBe(ADDRESS_ID)
		expect(stored.baseUrl).toBe('https://api.example.com/v1')
		const post = api.calls.find((call) => call.method === 'post')
		expect(post?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				title: 'API Adresse',
				version: '1.1.0',
				description: 'Addresses',
				base_url: 'https://api.example.com/v1',
				source: { kind: 'file', file_name: 'adresse.json' },
				security: [
					{
						name: 'cle',
						kind: {
							type: 'api_key',
							location: 'header',
							parameter_name: 'X-Key',
						},
					},
				],
				operations: [
					{
						id: 'createAddress',
						method: 'POST',
						path: '/addresses',
						summary: 'Create',
						tag: 'Addresses',
						parameters: [],
						request_body: { schema: { type: 'object' } },
						response: null,
						response_status: '201',
					},
				],
				schemas: {},
			},
		})
		await waitFor(() => expect(fetched(COLLECTIONS_PATH)).toBe(2))
	})

	it('puts the replacement on the stored collection and refetches the list and the detail', async () => {
		const { api, fetched } = installCollectionsApi()
		const { result } = renderHook(
			() => ({
				list: useCollections('org-1'),
				detail: useCollection('org-1', ADDRESS_ID),
				replace: useReplaceCollection('org-1'),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => {
			expect(result.current.list.isSuccess).toBe(true)
			expect(result.current.detail.isSuccess).toBe(true)
		})

		await act(() =>
			result.current.replace.mutateAsync({
				collectionId: ADDRESS_ID,
				draft: DRAFT,
			}),
		)

		const put = api.calls.find((call) => call.method === 'put')
		expect(put?.params).toMatchObject({
			path: { organization_id: 'org-1', collection_id: ADDRESS_ID },
			body: { version: '1.1.0', base_url: 'https://api.example.com/v1' },
		})
		await waitFor(() => {
			expect(fetched(COLLECTIONS_PATH)).toBe(2)
			expect(fetched(COLLECTION_PATH)).toBe(2)
		})
	})

	it('deletes a collection and refetches the list', async () => {
		const { api, fetched } = installCollectionsApi()
		const { result } = renderHook(
			() => ({
				list: useCollections('org-1'),
				remove: useDeleteCollection('org-1'),
			}),
			{ wrapper: stableWrapper() },
		)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		await act(() => result.current.remove.mutateAsync(COMPANY_ID))

		const call = api.calls.find((item) => item.method === 'delete')
		expect(call?.params).toEqual({
			path: { organization_id: 'org-1', collection_id: COMPANY_ID },
		})
		await waitFor(() => expect(fetched(COLLECTIONS_PATH)).toBe(2))
	})

	it('keeps the status of a refusal so the dialog can explain it', async () => {
		const { api } = installCollectionsApi()
		api.mockMutation('post', COLLECTIONS_PATH, () => {
			throw Object.assign(new Error('too large'), { status: 413 })
		})
		const { result } = renderHook(() => useImportCollection('org-1'), {
			wrapper: stableWrapper(),
		})

		await act(async () => {
			await result.current.mutateAsync(DRAFT).catch(() => undefined)
		})

		await waitFor(() => expect(result.current.isError).toBe(true))
		expect(result.current.error).toMatchObject({ status: 413 })
	})
})
