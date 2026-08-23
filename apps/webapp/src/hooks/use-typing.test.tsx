import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { GatewayProvider } from '#/hooks/use-gateway'
import { useSendTyping, useTypingUsers } from '#/hooks/use-typing'
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

function queryWrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	)
}

function gatewayWrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>
			<GatewayProvider>{children}</GatewayProvider>
		</QueryClientProvider>
	)
}

let postSpy: ReturnType<typeof vi.fn>

beforeEach(() => {
	FakeWebSocket.instances = []
	vi.stubGlobal('WebSocket', FakeWebSocket)
	postSpy = vi.fn().mockResolvedValue({ status: 204, data: undefined })
	window.tanstackApi = new TanstackQueryApiClient({ post: postSpy } as never)
	clearAuth()
})

describe('useSendTyping', () => {
	it('notifies the server on the first call', async () => {
		const { result } = renderHook(() => useSendTyping('ch-1'), {
			wrapper: queryWrapper,
		})

		act(() => result.current())

		// react-query dispatches through its own async retryer — the
		// underlying `client.post` call lands a microtask after `mutate()`.
		await waitFor(() => {
			expect(postSpy).toHaveBeenCalledWith(
				'/api/v1/chat/channels/{channel_id}/typing',
				expect.objectContaining({ path: { channel_id: 'ch-1' } }),
			)
		})
	})

	it('does not re-notify within the debounce window', async () => {
		const { result } = renderHook(() => useSendTyping('ch-1'), {
			wrapper: queryWrapper,
		})

		act(() => {
			result.current()
			result.current()
			result.current()
		})

		await waitFor(() => expect(postSpy).toHaveBeenCalledTimes(1))
	})
})

describe('useTypingUsers', () => {
	afterEach(() => {
		vi.useRealTimers()
	})

	it('adds a user on TYPING_START and expires them after ttl_ms', () => {
		vi.useFakeTimers()
		const { result } = renderHook(() => useTypingUsers('ch-1'), {
			wrapper: gatewayWrapper,
		})

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'TYPING_START',
				d: {
					organization_id: 'org-1',
					channel_id: 'ch-1',
					user_id: 'alice',
					ttl_ms: 10_000,
				},
			})
		})

		expect(result.current.has('alice')).toBe(true)

		act(() => {
			vi.advanceTimersByTime(10_000)
		})

		expect(result.current.has('alice')).toBe(false)
	})

	it('ignores typing events from another channel', () => {
		const { result } = renderHook(() => useTypingUsers('ch-1'), {
			wrapper: gatewayWrapper,
		})

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'TYPING_START',
				d: {
					organization_id: 'org-1',
					channel_id: 'ch-2',
					user_id: 'alice',
					ttl_ms: 10_000,
				},
			})
		})

		expect(result.current.size).toBe(0)
	})

	it('clears typing users when the channel changes', () => {
		const { result, rerender } = renderHook(
			({ channelId }) => useTypingUsers(channelId),
			{ wrapper: gatewayWrapper, initialProps: { channelId: 'ch-1' } },
		)

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'TYPING_START',
				d: {
					organization_id: 'org-1',
					channel_id: 'ch-1',
					user_id: 'alice',
					ttl_ms: 10_000,
				},
			})
		})
		expect(result.current.has('alice')).toBe(true)

		act(() => rerender({ channelId: 'ch-2' }))

		expect(result.current.size).toBe(0)
	})
})
