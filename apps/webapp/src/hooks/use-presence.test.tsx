import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { GatewayProvider } from '#/hooks/use-gateway'
import {
	PresenceProvider,
	useOnlineCount,
	usePresenceStatus,
} from '#/hooks/use-presence'
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
	// Separate boundaries: the socket only exists once React has flushed the
	// effect reacting to `setAuth`, and `PresenceProvider`'s own "publish on
	// connect" effect only fires once React has flushed the state update
	// `ready` causes — each needs its own `act()` to be observable next.
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

let putSpy: ReturnType<typeof vi.fn>

function wrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>
			<GatewayProvider>
				<PresenceProvider organizationId="org-1">{children}</PresenceProvider>
			</GatewayProvider>
		</QueryClientProvider>
	)
}

beforeEach(() => {
	FakeWebSocket.instances = []
	vi.stubGlobal('WebSocket', FakeWebSocket)
	Object.defineProperty(document, 'hidden', {
		configurable: true,
		get: () => false,
	})
	putSpy = vi.fn().mockResolvedValue({
		status: 200,
		data: {
			data: { organization_id: 'org-1', user_id: 'me', status: 'ONLINE' },
		},
	})
	window.tanstackApi = new TanstackQueryApiClient({ put: putSpy } as never)
	clearAuth()
})

describe('PresenceProvider', () => {
	it('publishes its own presence as ONLINE once the gateway connects', async () => {
		renderHook(() => usePresenceStatus('me'), { wrapper })

		connectAndBecomeReady()

		// react-query dispatches the mutation through its own async retryer,
		// so the underlying `client.put` call lands a microtask after `mutate()`
		// returns — a synchronous assertion right after would race it.
		await waitFor(() => {
			expect(putSpy).toHaveBeenCalledWith(
				'/api/v1/chat/organizations/{organization_id}/presence',
				expect.objectContaining({
					path: { organization_id: 'org-1' },
					body: { status: 'ONLINE' },
				}),
			)
		})
	})

	it('tracks another member going online from a gateway event', () => {
		const { result } = renderHook(() => usePresenceStatus('alice'), {
			wrapper,
		})
		expect(result.current).toBe('UNKNOWN')

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'PRESENCE_UPDATE',
				d: {
					organization_id: 'org-1',
					user_id: 'alice',
					status: 'ONLINE',
					updated_at: '2026-01-01T00:00:00Z',
				},
			})
		})

		expect(result.current).toBe('ONLINE')
	})

	it('ignores presence updates scoped to a different organization', () => {
		const { result } = renderHook(() => usePresenceStatus('alice'), {
			wrapper,
		})

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'PRESENCE_UPDATE',
				d: {
					organization_id: 'org-other',
					user_id: 'alice',
					status: 'ONLINE',
					updated_at: '2026-01-01T00:00:00Z',
				},
			})
		})

		expect(result.current).toBe('UNKNOWN')
	})

	it('counts online members', () => {
		const { result } = renderHook(() => useOnlineCount(), { wrapper })

		connectAndBecomeReady()
		act(() => {
			lastSocket().receive({
				op: 'dispatch',
				t: 'PRESENCE_UPDATE',
				d: { organization_id: 'org-1', user_id: 'alice', status: 'ONLINE' },
			})
			lastSocket().receive({
				op: 'dispatch',
				t: 'PRESENCE_UPDATE',
				d: { organization_id: 'org-1', user_id: 'bob', status: 'OFFLINE' },
			})
		})

		// self (me) publishes ONLINE on connect too, but that only updates the
		// server — the local map only reflects events received back over the wire.
		expect(result.current).toBe(1)
	})

	it('republishes OFFLINE when the tab is hidden, ONLINE when visible again', async () => {
		renderHook(() => usePresenceStatus('me'), { wrapper })
		connectAndBecomeReady()
		await waitFor(() => expect(putSpy).toHaveBeenCalled())
		putSpy.mockClear()

		act(() => {
			Object.defineProperty(document, 'hidden', {
				configurable: true,
				get: () => true,
			})
			document.dispatchEvent(new Event('visibilitychange'))
		})
		await waitFor(() => {
			expect(putSpy).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({ body: { status: 'OFFLINE' } }),
			)
		})

		putSpy.mockClear()
		act(() => {
			Object.defineProperty(document, 'hidden', {
				configurable: true,
				get: () => false,
			})
			document.dispatchEvent(new Event('visibilitychange'))
		})
		await waitFor(() => {
			expect(putSpy).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({ body: { status: 'ONLINE' } }),
			)
		})
	})

	it('publishes OFFLINE on unmount', async () => {
		const { unmount } = renderHook(() => usePresenceStatus('me'), {
			wrapper,
		})
		connectAndBecomeReady()
		await waitFor(() => expect(putSpy).toHaveBeenCalled())
		putSpy.mockClear()

		unmount()

		await waitFor(() => {
			expect(putSpy).toHaveBeenCalledWith(
				expect.anything(),
				expect.objectContaining({ body: { status: 'OFFLINE' } }),
			)
		})
	})
})
