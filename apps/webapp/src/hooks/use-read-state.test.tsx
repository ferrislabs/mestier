import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { useReadStateGatewaySync, useUnreadChannels } from '#/hooks/use-chat'
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
	close() {}
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

function connectAndBecomeReady() {
	act(() => {
		setAuth({ accessToken: 'tok', isAuthenticated: true })
	})
	act(() => {
		lastSocket().open()
	})
	act(() => {
		lastSocket().receive({ op: 'ready', user_id: 'me' })
	})
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

function unreadKey(organizationId: string) {
	return window.tanstackApi.get(
		'/api/v1/chat/organizations/{organization_id}/unread',
		{ path: { organization_id: organizationId } },
	).queryKey
}

function notificationsKey(organizationId: string) {
	return window.tanstackApi.get(
		'/api/v1/chat/organizations/{organization_id}/notifications',
		{
			path: {
				organization_id: organizationId,
				unread_only: null,
				before: null,
				limit: null,
			},
		} as never,
	).queryKey
}

beforeEach(() => {
	FakeWebSocket.instances = []
	vi.stubGlobal('WebSocket', FakeWebSocket)
	window.tanstackApi = new TanstackQueryApiClient({} as never)
	clearAuth()
})

describe('useUnreadChannels', () => {
	it('is not wrapped in a DataEnvelope — the endpoint returns the plain resource', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		// `read_state/unread.rs` returns `Json<UnreadResponse>` directly, not
		// through `Response<T>` — no `{ data, pagination }` wrapper here,
		// unlike every list endpoint elsewhere in this file.
		queryClient.setQueryData(unreadKey('org-1'), {
			channel_ids: ['ch-1', 'ch-2'],
		})

		const { result } = renderHook(() => useUnreadChannels('org-1'), {
			wrapper: createWrapper(queryClient),
		})

		expect(result.current.data).toEqual(new Set(['ch-1', 'ch-2']))
	})
})

describe('useReadStateGatewaySync', () => {
	it('clears a channel from the unread set on its own CHANNEL_READ echo', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(unreadKey('org-1'), {
			channel_ids: ['ch-1', 'ch-2'],
		})

		renderHook(() => useReadStateGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})
		connectAndBecomeReady()

		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'CHANNEL_READ',
				d: {
					organization_id: 'org-1',
					channel_id: 'ch-1',
					user_id: 'me',
					last_read_message_id: 'm-1',
				},
			})
		})

		expect(
			queryClient.getQueryData<{ channel_ids: string[] }>(unreadKey('org-1')),
		).toEqual({ channel_ids: ['ch-2'] })
	})

	it('adds a notification and marks its channel unread on NOTIFICATION_CREATE', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(unreadKey('org-1'), { channel_ids: [] })
		queryClient.setQueryData(notificationsKey('org-1'), { data: [] })

		renderHook(() => useReadStateGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})
		connectAndBecomeReady()

		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'NOTIFICATION_CREATE',
				d: {
					id: 'n-1',
					organization_id: 'org-1',
					user_id: 'me',
					channel_id: 'ch-1',
					message_id: 'm-1',
					kind: 'MENTION',
					read_at: null,
					created_at: '2026-01-01T00:00:00Z',
				},
			})
		})

		expect(
			queryClient.getQueryData<{ channel_ids: string[] }>(unreadKey('org-1')),
		).toEqual({ channel_ids: ['ch-1'] })
		expect(
			queryClient.getQueryData<{ data: unknown[] }>(notificationsKey('org-1')),
		).toEqual({
			data: [expect.objectContaining({ id: 'n-1', kind: 'MENTION' })],
		})
	})

	it('ignores events scoped to a different organization', () => {
		const queryClient = new QueryClient({
			defaultOptions: { queries: { staleTime: Number.POSITIVE_INFINITY } },
		})
		queryClient.setQueryData(unreadKey('org-1'), { channel_ids: ['ch-1'] })

		renderHook(() => useReadStateGatewaySync('org-1'), {
			wrapper: createWrapper(queryClient),
		})
		connectAndBecomeReady()

		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'CHANNEL_READ',
				d: {
					organization_id: 'org-other',
					channel_id: 'ch-1',
					user_id: 'me',
					last_read_message_id: 'm-1',
				},
			})
		})

		expect(
			queryClient.getQueryData<{ channel_ids: string[] }>(unreadKey('org-1')),
		).toEqual({ channel_ids: ['ch-1'] })
	})
})
