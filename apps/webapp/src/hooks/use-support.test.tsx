import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	SUPPORT_TICKETS_PER_PAGE,
	useMoveSupportTicket,
	useSupportSettings,
	useSupportTickets,
} from '#/hooks/use-support'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import type { SupportFilters } from '#/pages/support/types'

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const TICKET_STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'

function ticket(
	id: string,
	status: Schemas.TicketResponse['status'],
): Schemas.TicketResponse {
	return {
		id,
		number: 1,
		subject: `Demande ${id}`,
		body: 'Bonjour',
		status,
		assignee_member_id: null,
		assigned_at: null,
		custom_fields: {},
		badges: [],
		external_id: null,
		created_at: '2026-10-01T07:00:00Z',
		updated_at: '2026-10-01T07:00:00Z',
	}
}

const PAGINATION = { total: 240 }

function page(tickets: Schemas.TicketResponse[]) {
	return { data: tickets, pagination: PAGINATION }
}

interface Deferred<T> {
	promise: Promise<T>
	resolve: (value: T) => void
	reject: (reason: unknown) => void
}

function deferred<T>(): Deferred<T> {
	let resolve: (value: T) => void = () => {}
	let reject: (reason: unknown) => void = () => {}
	const promise = new Promise<T>((resolvePromise, rejectPromise) => {
		resolve = resolvePromise
		reject = rejectPromise
	})
	return { promise, resolve, reject }
}

type Handler = (params: unknown) => unknown

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

function installFakeTanstackApi() {
	const calls: RecordedCall[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	const fakeApi = {
		get(path: string, params: unknown) {
			const request = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: request.path, query: request.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						const handler = mutationHandlers.get(`${method}:${path}`)
						if (!handler) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return handler(params)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return {
		calls,
		mockGet(path: string, handler: Handler) {
			getHandlers.set(path, handler)
		},
		mockMutation(method: string, path: string, handler: Handler) {
			mutationHandlers.set(`${method}:${path}`, handler)
		},
	}
}

function renderWithClient<Result>(callback: () => Result) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}
	return { queryClient, ...renderHook(callback, { wrapper }) }
}

function statusesOf(data: { data: Schemas.TicketResponse[] } | undefined) {
	return data?.data.map((entry) => `${entry.id}:${entry.status}`)
}

function moveVariables(ticketId: string, status: string) {
	return {
		path: { organization_id: 'org-1', ticket_id: ticketId },
		body: { status },
	}
}

describe('SUPPORT_TICKETS_PER_PAGE', () => {
	it('is the largest page the API serves', () => {
		expect(SUPPORT_TICKETS_PER_PAGE).toBe(100)
	})
})

describe('useSupportTickets', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
	})

	it('lists the tickets of the organization with the filters as query', async () => {
		api.mockGet(TICKETS_PATH, () => page([ticket('a', 'new')]))
		const filters: SupportFilters = {
			...EMPTY_SUPPORT_FILTERS,
			assignee: 'member-1',
			custom_field_key: 'ville',
		}

		const { result } = renderWithClient(() =>
			useSupportTickets('org-1', filters),
		)

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(statusesOf(result.current.data)).toEqual(['a:new'])
		expect(api.calls).toEqual([
			{
				method: 'get',
				path: TICKETS_PATH,
				params: {
					path: { organization_id: 'org-1' },
					query: { per_page: 100, assignee: 'member-1' },
				},
			},
		])
	})

	it('does not fetch without an organization', async () => {
		api.mockGet(TICKETS_PATH, () => page([]))

		const { result } = renderWithClient(() =>
			useSupportTickets('', EMPTY_SUPPORT_FILTERS),
		)

		await act(async () => {
			await Promise.resolve()
		})
		expect(result.current.fetchStatus).toBe('idle')
		expect(api.calls).toEqual([])
	})
})

describe('useSupportSettings', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
	})

	it('reads the display fields of the organization', async () => {
		api.mockGet(SETTINGS_PATH, () => ({
			data: { display_fields: ['ville', 'etage'] },
		}))

		const { result } = renderWithClient(() => useSupportSettings('org-1'))

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data.display_fields).toEqual(['ville', 'etage'])
		expect(api.calls).toEqual([
			{
				method: 'get',
				path: SETTINGS_PATH,
				params: { path: { organization_id: 'org-1' } },
			},
		])
	})

	it('does not fetch without an organization', async () => {
		api.mockGet(SETTINGS_PATH, () => ({ data: { display_fields: [] } }))

		const { result } = renderWithClient(() => useSupportSettings(''))

		await act(async () => {
			await Promise.resolve()
		})
		expect(result.current.fetchStatus).toBe('idle')
		expect(api.calls).toEqual([])
	})
})

describe('useMoveSupportTicket', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
	})

	function renderBoard(filters: SupportFilters = EMPTY_SUPPORT_FILTERS) {
		return renderWithClient(() => ({
			list: useSupportTickets('org-1', filters),
			move: useMoveSupportTicket('org-1', filters),
		}))
	}

	it('rewrites the cached list at once and keeps the pagination', async () => {
		const server = [ticket('a', 'new'), ticket('b', 'new'), ticket('c', 'new')]
		api.mockGet(TICKETS_PATH, () => page(server))
		const patch = deferred<unknown>()
		api.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise)

		const { result } = renderBoard()
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		act(() => {
			result.current.move.mutate(moveVariables('b', 'in_progress'))
		})

		await waitFor(() =>
			expect(statusesOf(result.current.list.data)).toEqual([
				'a:new',
				'b:in_progress',
				'c:new',
			]),
		)
		expect(result.current.list.data?.pagination).toEqual(PAGINATION)
		expect(api.calls.filter((call) => call.method === 'patch')).toEqual([
			{
				method: 'patch',
				path: TICKET_STATUS_PATH,
				params: moveVariables('b', 'in_progress'),
			},
		])

		patch.resolve({ data: ticket('b', 'in_progress') })
		await waitFor(() => expect(result.current.move.isSuccess).toBe(true))
	})

	it('writes into the cache entry of the filtered list it was given', async () => {
		const filters: SupportFilters = {
			...EMPTY_SUPPORT_FILTERS,
			assignee: 'member-1',
		}
		api.mockGet(TICKETS_PATH, () => page([ticket('a', 'new')]))
		const patch = deferred<unknown>()
		api.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise)

		const { result } = renderBoard(filters)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		act(() => {
			result.current.move.mutate(moveVariables('a', 'resolved'))
		})

		await waitFor(() =>
			expect(statusesOf(result.current.list.data)).toEqual(['a:resolved']),
		)

		patch.resolve({ data: ticket('a', 'resolved') })
		await waitFor(() => expect(result.current.move.isSuccess).toBe(true))
	})

	it('restores the previous page whole when the PATCH is rejected', async () => {
		const server = [ticket('a', 'new'), ticket('b', 'new'), ticket('c', 'new')]
		const refetch = deferred<unknown>()
		let reads = 0
		api.mockGet(TICKETS_PATH, () => {
			reads += 1
			return reads === 1 ? page(server) : refetch.promise
		})
		const patch = deferred<unknown>()
		api.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise)

		const { result } = renderBoard()
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))
		const before = result.current.list.data

		act(() => {
			result.current.move.mutate(moveVariables('b', 'resolved'))
		})
		await waitFor(() =>
			expect(statusesOf(result.current.list.data)).toEqual([
				'a:new',
				'b:resolved',
				'c:new',
			]),
		)

		patch.reject(new Error('refused'))

		await waitFor(() => expect(reads).toBe(2))
		expect(result.current.list.data).toEqual(before)
		expect(statusesOf(result.current.list.data)).toEqual([
			'a:new',
			'b:new',
			'c:new',
		])
		expect(result.current.list.data?.pagination).toEqual(PAGINATION)

		refetch.resolve(page(server))
		await waitFor(() => expect(result.current.move.isError).toBe(true))
	})

	it('refetches the list once the PATCH has settled', async () => {
		let reads = 0
		api.mockGet(TICKETS_PATH, () => {
			reads += 1
			return reads === 1
				? page([ticket('a', 'new')])
				: page([ticket('a', 'in_progress')])
		})
		api.mockMutation('patch', TICKET_STATUS_PATH, () => ({
			data: ticket('a', 'in_progress'),
		}))

		const { result } = renderBoard()
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		act(() => {
			result.current.move.mutate(moveVariables('a', 'in_progress'))
		})

		await waitFor(() => expect(reads).toBe(2))
		await waitFor(() =>
			expect(statusesOf(result.current.list.data)).toEqual(['a:in_progress']),
		)
	})
})
