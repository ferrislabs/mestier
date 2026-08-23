import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { useChannels, useChatListGatewaySync } from '#/hooks/use-chat'
import { GatewayProvider } from '#/hooks/use-gateway'
import { clearAuth, setAuth } from '#/store/auth.store'

class FakeWebSocket {
	static instances: FakeWebSocket[] = []
	onopen: (() => void) | null = null
	onmessage: ((event: { data: string }) => void) | null = null
	onclose: (() => void) | null = null
	onerror: (() => void) | null = null

	constructor(public url: string) {
		FakeWebSocket.instances.push(this)
	}

	send() {}
	close() {
		this.onclose?.()
	}
	open() {
		this.onopen?.()
	}
	receive(message: unknown) {
		this.onmessage?.({ data: JSON.stringify(message) })
	}
}

function lastSocket(): FakeWebSocket {
	const socket = FakeWebSocket.instances.at(-1)
	if (!socket) throw new Error('no socket created')
	return socket
}

function channel(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		id: 'ch-1',
		organization_id: 'org-1',
		channel_type: 'TEXT',
		name: 'general',
		topic: null,
		position: 0,
		category_id: null,
		parent_id: null,
		origin_message_id: null,
		archived: false,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function createWrapper(queryClient: QueryClient) {
	return function Wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<GatewayProvider>{children}</GatewayProvider>
			</QueryClientProvider>
		)
	}
}

beforeEach(() => {
	FakeWebSocket.instances = []
	vi.stubGlobal('WebSocket', FakeWebSocket)
	window.tanstackApi = new TanstackQueryApiClient({} as never)
	clearAuth()
})

function channelsKey(organizationId: string) {
	return window.tanstackApi.get(
		'/api/v1/chat/organizations/{organization_id}/channels',
		{
			path: { organization_id: organizationId },
		},
	).queryKey
}

describe('useChannels', () => {
	it('filters threads out of the organization channel list', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(channelsKey('org-1'), [
			channel({ id: 'ch-1', channel_type: 'TEXT' }),
			channel({ id: 'ch-2', channel_type: 'THREAD', parent_id: 'ch-1' }),
		])

		const { result } = renderHook(() => useChannels('org-1'), {
			wrapper: createWrapper(queryClient),
		})

		expect(result.current.data?.map((c) => c.id)).toEqual(['ch-1'])
	})
})

describe('useChatListGatewaySync', () => {
	it('adds a channel created on another tab to the cached list', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(channelsKey('org-1'), [channel({ id: 'ch-1' })])

		renderHook(() => useChatListGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})

		act(() => {
			setAuth({ accessToken: 'tok', isAuthenticated: true })
		})
		act(() => {
			lastSocket().open()
			lastSocket().receive({ op: 'ready', user_id: 'user-1' })
			lastSocket().receive({
				op: 'dispatch',
				t: 'CHANNEL_CREATE',
				d: channel({ id: 'ch-2', name: 'random' }),
			})
		})

		expect(
			queryClient.getQueryData<{ id: string }[]>(channelsKey('org-1')),
		).toHaveLength(2)
	})

	it('removes a deleted channel from the cached list', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(channelsKey('org-1'), [
			channel({ id: 'ch-1' }),
			channel({ id: 'ch-2' }),
		])

		renderHook(() => useChatListGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})

		act(() => {
			setAuth({ accessToken: 'tok', isAuthenticated: true })
		})
		act(() => {
			lastSocket().open()
			lastSocket().receive({ op: 'ready', user_id: 'user-1' })
			lastSocket().receive({
				op: 'dispatch',
				t: 'CHANNEL_DELETE',
				d: { organization_id: 'org-1', channel_id: 'ch-2' },
			})
		})

		expect(
			queryClient.getQueryData<{ id: string }[]>(channelsKey('org-1')),
		).toEqual([channel({ id: 'ch-1' })])
	})

	it('ignores events scoped to a different organization', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(channelsKey('org-1'), [channel({ id: 'ch-1' })])

		renderHook(() => useChatListGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})

		act(() => {
			setAuth({ accessToken: 'tok', isAuthenticated: true })
		})
		act(() => {
			lastSocket().open()
			lastSocket().receive({ op: 'ready', user_id: 'user-1' })
			lastSocket().receive({
				op: 'dispatch',
				t: 'CHANNEL_CREATE',
				d: channel({ id: 'ch-9', organization_id: 'org-other' }),
			})
		})

		expect(
			queryClient.getQueryData<{ id: string }[]>(channelsKey('org-1')),
		).toHaveLength(1)
	})
})
