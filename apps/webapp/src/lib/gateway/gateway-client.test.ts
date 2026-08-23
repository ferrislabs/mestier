import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { GatewayClient, type WebSocketLike } from '#/lib/gateway/gateway-client'
import type { GatewayEvent } from '#/lib/gateway/wire-types'

/** Minimal fake matching the `WebSocketLike` surface `GatewayClient` needs. */
class FakeSocket {
	static instances: FakeSocket[] = []
	sent: string[] = []
	closed = false
	onopen: (() => void) | null = null
	onmessage: ((event: { data: string }) => void) | null = null
	onclose: (() => void) | null = null
	onerror: (() => void) | null = null

	constructor(public url: string) {
		FakeSocket.instances.push(this)
	}

	send(data: string) {
		this.sent.push(data)
	}

	close() {
		this.closed = true
		this.onclose?.()
	}

	open() {
		this.onopen?.()
	}

	receive(message: unknown) {
		this.onmessage?.({ data: JSON.stringify(message) })
	}
}

function lastSocket(): FakeSocket {
	const socket = FakeSocket.instances.at(-1)
	if (!socket) throw new Error('no socket created')
	return socket
}

function createClient(overrides: { getToken?: () => string | null } = {}) {
	return new GatewayClient({
		url: 'ws://localhost:3456/api/v1/chat/gateway',
		getToken: overrides.getToken ?? (() => 'tok-123'),
		wsFactory: (url) => new FakeSocket(url) as unknown as WebSocketLike,
		backoff: { baseMs: 1_000, maxMs: 30_000, jitterRatio: 0, random: () => 0 },
	})
}

beforeEach(() => {
	FakeSocket.instances = []
	vi.useFakeTimers()
})

afterEach(() => {
	vi.useRealTimers()
})

describe('GatewayClient', () => {
	it('sends identify as soon as the socket opens', () => {
		const client = createClient()
		client.connect()
		lastSocket().open()

		expect(lastSocket().sent).toEqual([
			JSON.stringify({ op: 'identify', token: 'tok-123' }),
		])
	})

	it('reaches the open state once ready is received', () => {
		const client = createClient()
		const states: string[] = []
		client.onStateChange((s) => states.push(s))

		client.connect()
		expect(states).toContain('connecting')
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })

		expect(client.getState()).toBe('open')
		expect(states.at(-1)).toBe('open')
	})

	it('acks a server heartbeat immediately', () => {
		const client = createClient()
		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })
		lastSocket().sent = []

		lastSocket().receive({ op: 'heartbeat' })

		expect(lastSocket().sent).toEqual([JSON.stringify({ op: 'heartbeat_ack' })])
	})

	it('dispatches typed events to subscribers', () => {
		const client = createClient()
		const received: GatewayEvent[] = []
		client.on('MESSAGE_CREATE', (event) => received.push(event))

		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })
		lastSocket().receive({
			op: 'dispatch',
			t: 'MESSAGE_CREATE',
			d: { id: 'm1', content: 'hey' },
		})

		expect(received).toHaveLength(1)
		expect(received[0]).toEqual({
			type: 'MESSAGE_CREATE',
			data: { id: 'm1', content: 'hey' },
		})
	})

	it('reconnects after an unexpected close, with a growing delay, and re-identifies', () => {
		const client = createClient()
		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })

		// First drop: unexpected close.
		lastSocket().onclose?.()
		expect(client.getState()).toBe('reconnecting')
		expect(FakeSocket.instances).toHaveLength(1)

		vi.advanceTimersByTime(1_000) // baseMs * 2^0, zero jitter
		expect(FakeSocket.instances).toHaveLength(2)
		lastSocket().open()
		expect(lastSocket().sent).toEqual([
			JSON.stringify({ op: 'identify', token: 'tok-123' }),
		])

		// Second drop before reaching `ready`: attempt counter must have grown.
		lastSocket().onclose?.()
		vi.advanceTimersByTime(1_999)
		expect(FakeSocket.instances).toHaveLength(2) // not yet — needs 2000ms (2^1)
		vi.advanceTimersByTime(1)
		expect(FakeSocket.instances).toHaveLength(3)
	})

	it('resets the reconnect attempt counter after reaching open again', () => {
		const client = createClient()
		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })

		lastSocket().onclose?.()
		vi.advanceTimersByTime(1_000)
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })

		lastSocket().onclose?.()
		vi.advanceTimersByTime(999)
		expect(FakeSocket.instances).toHaveLength(2) // still waiting on the 1000ms base delay again
		vi.advanceTimersByTime(1)
		expect(FakeSocket.instances).toHaveLength(3)
	})

	it('does not reconnect after an explicit disconnect', () => {
		const client = createClient()
		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })

		client.disconnect()
		expect(client.getState()).toBe('closed')

		vi.advanceTimersByTime(60_000)
		expect(FakeSocket.instances).toHaveLength(1)
	})

	it('stays idle when disconnected before ever connecting', () => {
		const client = createClient()
		expect(client.getState()).toBe('idle')

		client.disconnect()

		expect(client.getState()).toBe('idle')
		expect(FakeSocket.instances).toHaveLength(0)
	})

	it('does not attempt to connect without a token', () => {
		const client = createClient({ getToken: () => null })
		client.connect()
		expect(FakeSocket.instances).toHaveLength(0)
		expect(client.getState()).toBe('idle')
	})

	it('unsubscribes handlers', () => {
		const client = createClient()
		const received: GatewayEvent[] = []
		const unsubscribe = client.on('MESSAGE_CREATE', (event) =>
			received.push(event),
		)
		unsubscribe()

		client.connect()
		lastSocket().open()
		lastSocket().receive({ op: 'ready', user_id: 'user-1' })
		lastSocket().receive({ op: 'dispatch', t: 'MESSAGE_CREATE', d: {} })

		expect(received).toHaveLength(0)
	})
})
