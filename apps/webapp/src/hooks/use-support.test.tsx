import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import {
	afterEach,
	beforeEach,
	describe,
	expect,
	it,
	type MockInstance,
	vi,
} from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	SUPPORT_TICKETS_PER_PAGE,
	useAddSupportEntry,
	useAssignSupportTicket,
	useDeleteSupportDraft,
	useEditSupportDraft,
	useMoveSupportTicket,
	useOpenSupportAttachment,
	useSendSupportDraft,
	useSupportSettings,
	useSupportThread,
	useSupportTicket,
	useSupportTickets,
	useTransitionSupportTicket,
} from '#/hooks/use-support'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import type { SupportFilters } from '#/pages/support/types'

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const TICKET_STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'
const TICKET_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}'
const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries'
const ASSIGNEE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee'
const ENTRY_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}'
const ENTRY_SEND_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send'
const ATTACHMENT_URL_PATH =
	'/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url'
const SUPPORT_PREFIX = '/api/v1/organizations/{organization_id}/support'

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

function threadEntry(
	id: string,
	kind: Schemas.TicketEntryResponse['kind'] = 'reply',
): Schemas.TicketEntryResponse {
	return {
		id,
		ticket_id: 'ticket-1',
		kind,
		author_member_id: 'member-1',
		body: 'Nous passons demain.',
		attachments: [],
		created_at: '2026-10-01T07:00:00Z',
		updated_at: '2026-10-01T07:00:00Z',
	}
}

function readLabel(call: RecordedCall) {
	const path = (call.params as { path?: Record<string, string> }).path ?? {}
	return `${call.path.slice(SUPPORT_PREFIX.length)} ${Object.values(path).join('/')}`
}

function readCounts(calls: RecordedCall[]) {
	const counts: Record<string, number> = {}
	for (const call of calls) {
		if (call.method !== 'get') continue
		const label = readLabel(call)
		counts[label] = (counts[label] ?? 0) + 1
	}
	return counts
}

function refetchedSince(before: Record<string, number>, calls: RecordedCall[]) {
	const refetched: Record<string, number> = {}
	for (const [label, count] of Object.entries(readCounts(calls))) {
		const extra = count - (before[label] ?? 0)
		if (extra > 0) refetched[label] = extra
	}
	return refetched
}

const THREAD_LABEL = '/tickets/{ticket_id}/entries org-1/ticket-1'
const TICKET_LABEL = '/tickets/{ticket_id} org-1/ticket-1'
const LIST_LABEL = '/tickets org-1'

describe('useSupportTicket', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
	})

	it('reads one ticket of the organization', async () => {
		api.mockGet(TICKET_PATH, () => ({ data: ticket('ticket-1', 'new') }))

		const { result } = renderWithClient(() =>
			useSupportTicket('org-1', 'ticket-1'),
		)

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data.id).toBe('ticket-1')
		expect(api.calls).toEqual([
			{
				method: 'get',
				path: TICKET_PATH,
				params: { path: { organization_id: 'org-1', ticket_id: 'ticket-1' } },
			},
		])
	})

	it('does not fetch without an organization or without a ticket', async () => {
		api.mockGet(TICKET_PATH, () => ({ data: ticket('ticket-1', 'new') }))

		const { result } = renderWithClient(() => ({
			withoutOrganization: useSupportTicket('', 'ticket-1'),
			withoutTicket: useSupportTicket('org-1', ''),
		}))

		await act(async () => {
			await Promise.resolve()
		})
		expect(result.current.withoutOrganization.fetchStatus).toBe('idle')
		expect(result.current.withoutTicket.fetchStatus).toBe('idle')
		expect(api.calls).toEqual([])
	})
})

describe('useSupportThread', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
	})

	it('reads the entries of the ticket', async () => {
		api.mockGet(ENTRIES_PATH, () => ({
			data: [threadEntry('entry-1', 'client_message'), threadEntry('entry-2')],
		}))

		const { result } = renderWithClient(() =>
			useSupportThread('org-1', 'ticket-1'),
		)

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data.map((entry) => entry.id)).toEqual([
			'entry-1',
			'entry-2',
		])
		expect(api.calls).toEqual([
			{
				method: 'get',
				path: ENTRIES_PATH,
				params: { path: { organization_id: 'org-1', ticket_id: 'ticket-1' } },
			},
		])
	})

	it('does not fetch without an organization or without a ticket', async () => {
		api.mockGet(ENTRIES_PATH, () => ({ data: [] }))

		const { result } = renderWithClient(() => ({
			withoutOrganization: useSupportThread('', 'ticket-1'),
			withoutTicket: useSupportThread('org-1', ''),
		}))

		await act(async () => {
			await Promise.resolve()
		})
		expect(result.current.withoutOrganization.fetchStatus).toBe('idle')
		expect(result.current.withoutTicket.fetchStatus).toBe('idle')
		expect(api.calls).toEqual([])
	})
})

describe('support ticket page mutations', () => {
	let api: ReturnType<typeof installFakeTanstackApi>

	beforeEach(() => {
		api = installFakeTanstackApi()
		api.mockGet(TICKET_PATH, () => ({ data: ticket('ticket-1', 'new') }))
		api.mockGet(ENTRIES_PATH, () => ({ data: [threadEntry('entry-1')] }))
		api.mockGet(TICKETS_PATH, () => page([ticket('ticket-1', 'new')]))
		api.mockGet(SETTINGS_PATH, () => ({ data: { display_fields: [] } }))
	})

	async function renderTicketPage() {
		const rendered = renderWithClient(() => ({
			queries: [
				useSupportTicket('org-1', 'ticket-1'),
				useSupportThread('org-1', 'ticket-1'),
				useSupportTicket('org-1', 'ticket-2'),
				useSupportThread('org-1', 'ticket-2'),
				useSupportTicket('org-2', 'ticket-1'),
				useSupportThread('org-2', 'ticket-1'),
				useSupportTickets('org-1', EMPTY_SUPPORT_FILTERS),
				useSupportTickets('org-2', EMPTY_SUPPORT_FILTERS),
				useSupportSettings('org-1'),
			],
			add: useAddSupportEntry('org-1', 'ticket-1'),
			edit: useEditSupportDraft('org-1', 'ticket-1'),
			send: useSendSupportDraft('org-1', 'ticket-1'),
			remove: useDeleteSupportDraft('org-1', 'ticket-1'),
			transition: useTransitionSupportTicket('org-1', 'ticket-1'),
			assign: useAssignSupportTicket('org-1', 'ticket-1'),
		}))
		await waitFor(() =>
			expect(
				rendered.result.current.queries.every((query) => query.isSuccess),
			).toBe(true),
		)
		return rendered
	}

	it('adds an entry, then refreshes the thread and the ticket only', async () => {
		api.mockMutation('post', ENTRIES_PATH, () => ({
			data: threadEntry('entry-2'),
		}))
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = {
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'reply', body: 'Bonjour' },
		}

		act(() => {
			result.current.add.mutate(variables)
		})
		await waitFor(() => expect(result.current.add.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'post')).toEqual([
			{ method: 'post', path: ENTRIES_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[THREAD_LABEL]: 1,
			[TICKET_LABEL]: 1,
		})
	})

	it('sends a draft kind and a note kind as given', async () => {
		api.mockMutation('post', ENTRIES_PATH, () => ({
			data: threadEntry('entry-2', 'draft'),
		}))
		const { result } = await renderTicketPage()
		const path = { organization_id: 'org-1', ticket_id: 'ticket-1' }

		act(() => {
			result.current.add.mutate({ path, body: { kind: 'draft', body: 'A' } })
		})
		await waitFor(() => expect(result.current.add.isSuccess).toBe(true))
		act(() => {
			result.current.add.mutate({ path, body: { kind: 'note', body: 'B' } })
		})
		await waitFor(() =>
			expect(api.calls.filter((call) => call.method === 'post')).toHaveLength(
				2,
			),
		)

		expect(
			api.calls
				.filter((call) => call.method === 'post')
				.map((call) => (call.params as { body: unknown }).body),
		).toEqual([
			{ kind: 'draft', body: 'A' },
			{ kind: 'note', body: 'B' },
		])
	})

	it('refreshes the thread and the ticket even when adding is rejected', async () => {
		api.mockMutation('post', ENTRIES_PATH, () => {
			throw new Error('refused')
		})
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)

		act(() => {
			result.current.add.mutate({
				path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
				body: { kind: 'reply', body: 'Bonjour' },
			})
		})
		await waitFor(() => expect(result.current.add.isError).toBe(true))

		expect(refetchedSince(before, api.calls)).toEqual({
			[THREAD_LABEL]: 1,
			[TICKET_LABEL]: 1,
		})
	})

	it('edits a draft, then refreshes the thread and the ticket only', async () => {
		api.mockMutation('patch', ENTRY_PATH, () => ({
			data: threadEntry('entry-1', 'draft'),
		}))
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = {
			path: { organization_id: 'org-1', entry_id: 'entry-1' },
			body: { body: 'Texte corrigé' },
		}

		act(() => {
			result.current.edit.mutate(variables)
		})
		await waitFor(() => expect(result.current.edit.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'patch')).toEqual([
			{ method: 'patch', path: ENTRY_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[THREAD_LABEL]: 1,
			[TICKET_LABEL]: 1,
		})
	})

	it('sends a draft, then refreshes the thread and the ticket only', async () => {
		api.mockMutation('post', ENTRY_SEND_PATH, () => ({
			data: threadEntry('entry-1', 'reply'),
		}))
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = {
			path: { organization_id: 'org-1', entry_id: 'entry-1' },
		}

		act(() => {
			result.current.send.mutate(variables)
		})
		await waitFor(() => expect(result.current.send.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'post')).toEqual([
			{ method: 'post', path: ENTRY_SEND_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[THREAD_LABEL]: 1,
			[TICKET_LABEL]: 1,
		})
	})

	it('deletes a draft, then refreshes the thread and the ticket only', async () => {
		api.mockMutation('delete', ENTRY_PATH, () => undefined)
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = {
			path: { organization_id: 'org-1', entry_id: 'entry-1' },
		}

		act(() => {
			result.current.remove.mutate(variables)
		})
		await waitFor(() => expect(result.current.remove.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'delete')).toEqual([
			{ method: 'delete', path: ENTRY_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[THREAD_LABEL]: 1,
			[TICKET_LABEL]: 1,
		})
	})

	it('moves the status, then refreshes the ticket and the lists of the organization only', async () => {
		api.mockMutation('patch', TICKET_STATUS_PATH, () => ({
			data: ticket('ticket-1', 'resolved'),
		}))
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = moveVariables('ticket-1', 'resolved')

		act(() => {
			result.current.transition.mutate(variables)
		})
		await waitFor(() => expect(result.current.transition.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'patch')).toEqual([
			{ method: 'patch', path: TICKET_STATUS_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[TICKET_LABEL]: 1,
			[LIST_LABEL]: 1,
		})
	})

	it('refreshes the ticket and the lists even when the transition is rejected', async () => {
		api.mockMutation('patch', TICKET_STATUS_PATH, () => {
			throw new Error('refused')
		})
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)

		act(() => {
			result.current.transition.mutate(moveVariables('ticket-1', 'resolved'))
		})
		await waitFor(() => expect(result.current.transition.isError).toBe(true))

		expect(refetchedSince(before, api.calls)).toEqual({
			[TICKET_LABEL]: 1,
			[LIST_LABEL]: 1,
		})
	})

	it('assigns a member, then refreshes the ticket and the lists of the organization only', async () => {
		api.mockMutation('put', ASSIGNEE_PATH, () => ({
			data: { ...ticket('ticket-1', 'new'), assignee_member_id: 'member-1' },
		}))
		const { result } = await renderTicketPage()
		const before = readCounts(api.calls)
		const variables = {
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: 'member-1' },
		}

		act(() => {
			result.current.assign.mutate(variables)
		})
		await waitFor(() => expect(result.current.assign.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'put')).toEqual([
			{ method: 'put', path: ASSIGNEE_PATH, params: variables },
		])
		expect(refetchedSince(before, api.calls)).toEqual({
			[TICKET_LABEL]: 1,
			[LIST_LABEL]: 1,
		})
	})

	it('unassigns with a null member', async () => {
		api.mockMutation('put', ASSIGNEE_PATH, () => ({
			data: ticket('ticket-1', 'new'),
		}))
		const { result } = await renderTicketPage()
		const variables = {
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: null },
		}

		act(() => {
			result.current.assign.mutate(variables)
		})
		await waitFor(() => expect(result.current.assign.isSuccess).toBe(true))

		expect(api.calls.filter((call) => call.method === 'put')).toEqual([
			{ method: 'put', path: ASSIGNEE_PATH, params: variables },
		])
	})

	it('does not touch the cache optimistically while the request is pending', async () => {
		const patch = deferred<unknown>()
		api.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise)
		const { result } = await renderTicketPage()
		const before = result.current.queries[0]?.data

		act(() => {
			result.current.transition.mutate(moveVariables('ticket-1', 'resolved'))
		})
		await waitFor(() =>
			expect(api.calls.some((call) => call.method === 'patch')).toBe(true),
		)

		expect(result.current.queries[0]?.data).toEqual(before)
		expect(result.current.queries[6]?.data).toEqual(
			page([ticket('ticket-1', 'new')]),
		)

		patch.resolve({ data: ticket('ticket-1', 'resolved') })
		await waitFor(() => expect(result.current.transition.isSuccess).toBe(true))
	})
})

describe('useOpenSupportAttachment', () => {
	let api: ReturnType<typeof installFakeTanstackApi>
	let openSpy: MockInstance<typeof window.open>

	beforeEach(() => {
		api = installFakeTanstackApi()
		openSpy = vi.spyOn(window, 'open').mockImplementation(() => null)
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	function signedUrl(url: string) {
		return { data: { url, expires_at: '2026-10-01T07:05:00Z' } }
	}

	it('fetches and opens nothing on render', async () => {
		api.mockGet(ATTACHMENT_URL_PATH, () => signedUrl('https://files.test/a'))

		renderWithClient(() => useOpenSupportAttachment('org-1'))

		await act(async () => {
			await Promise.resolve()
		})
		expect(api.calls).toEqual([])
		expect(openSpy).not.toHaveBeenCalled()
	})

	it('opens the signed url in a new tab once the click-time fetch resolved', async () => {
		const signed = deferred<unknown>()
		api.mockGet(ATTACHMENT_URL_PATH, () => signed.promise)
		const { result } = renderWithClient(() => useOpenSupportAttachment('org-1'))

		let opening: Promise<void> = Promise.resolve()
		act(() => {
			opening = result.current('attachment-1')
		})

		await waitFor(() => expect(api.calls).toHaveLength(1))
		expect(api.calls).toEqual([
			{
				method: 'get',
				path: ATTACHMENT_URL_PATH,
				params: {
					path: { organization_id: 'org-1', attachment_id: 'attachment-1' },
				},
			},
		])
		expect(openSpy).not.toHaveBeenCalled()

		signed.resolve(signedUrl('https://files.test/a'))
		await act(async () => {
			await opening
		})

		expect(openSpy).toHaveBeenCalledTimes(1)
		expect(openSpy).toHaveBeenCalledWith(
			'https://files.test/a',
			'_blank',
			'noopener,noreferrer',
		)
	})

	it('fetches a fresh link on every click', async () => {
		let reads = 0
		api.mockGet(ATTACHMENT_URL_PATH, () => {
			reads += 1
			return signedUrl(`https://files.test/a?signature=${reads}`)
		})
		const { result } = renderWithClient(() => useOpenSupportAttachment('org-1'))

		await act(async () => {
			await result.current('attachment-1')
		})
		await act(async () => {
			await result.current('attachment-1')
		})

		expect(api.calls).toHaveLength(2)
		expect(openSpy.mock.calls.map((call) => call[0])).toEqual([
			'https://files.test/a?signature=1',
			'https://files.test/a?signature=2',
		])
	})

	it('rejects and opens nothing when the fetch fails', async () => {
		api.mockGet(ATTACHMENT_URL_PATH, () => {
			throw new Error('expired')
		})
		const { result } = renderWithClient(() => useOpenSupportAttachment('org-1'))

		await act(async () => {
			await expect(result.current('attachment-1')).rejects.toThrow('expired')
		})

		expect(openSpy).not.toHaveBeenCalled()
	})
})
