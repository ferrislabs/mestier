import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { GatewayProvider, useGatewayConnectionState } from '#/hooks/use-gateway'
import { authStore, clearAuth, setAuth } from '#/store/auth.store'

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
}

beforeEach(() => {
	FakeWebSocket.instances = []
	vi.stubGlobal('WebSocket', FakeWebSocket)
	clearAuth()
})

describe('GatewayProvider', () => {
	it('stays idle before authentication', () => {
		const { result } = renderHook(() => useGatewayConnectionState(), {
			wrapper: GatewayProvider,
		})

		expect(result.current).toBe('idle')
		expect(FakeWebSocket.instances).toHaveLength(0)
	})

	it('connects once the session authenticates', () => {
		const { result } = renderHook(() => useGatewayConnectionState(), {
			wrapper: GatewayProvider,
		})

		act(() => {
			setAuth({ accessToken: 'tok-123', isAuthenticated: true })
		})

		expect(FakeWebSocket.instances).toHaveLength(1)
		expect(result.current).toBe('connecting')
	})

	it('disconnects when the session is cleared', () => {
		const { result } = renderHook(() => useGatewayConnectionState(), {
			wrapper: GatewayProvider,
		})

		act(() => {
			setAuth({ accessToken: 'tok-123', isAuthenticated: true })
		})
		act(() => {
			clearAuth()
		})

		expect(result.current).toBe('closed')
	})

	it('exposes the token to the socket via identify, taken from the auth store', () => {
		renderHook(() => useGatewayConnectionState(), { wrapper: GatewayProvider })

		act(() => {
			setAuth({ accessToken: 'secret-token', isAuthenticated: true })
		})
		const socket = FakeWebSocket.instances[0]
		expect(socket).toBeDefined()
		expect(authStore.state.accessToken).toBe('secret-token')
	})
})
