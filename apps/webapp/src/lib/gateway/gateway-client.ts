import { type BackoffOptions, computeBackoffMs } from '#/lib/gateway/backoff'
import {
	type ClientMessage,
	type GatewayEvent,
	type GatewayEventType,
	isGatewayEventType,
	type ServerMessage,
} from '#/lib/gateway/wire-types'

export type GatewayConnectionState =
	| 'idle'
	| 'connecting'
	| 'open'
	| 'reconnecting'
	| 'closed'

/** The slice of the browser `WebSocket` surface the client depends on — kept
 * narrow so tests can inject a fake without implementing the whole API. */
export interface WebSocketLike {
	onopen: (() => void) | null
	onmessage: ((event: { data: string }) => void) | null
	onclose: (() => void) | null
	onerror: (() => void) | null
	send(data: string): void
	close(): void
}

export interface GatewayClientOptions {
	url: string
	getToken: () => string | null
	wsFactory?: (url: string) => WebSocketLike
	backoff?: Partial<BackoffOptions>
}

type StateHandler = (state: GatewayConnectionState) => void
type EventHandler = (event: GatewayEvent) => void

/**
 * One reconnecting WebSocket connection to `/api/v1/chat/gateway`, per the
 * protocol in `gateway.rs`: identify within 10s of opening, ack every
 * server-sent heartbeat, reconnect with backoff on any unexpected close.
 *
 * Owned by a single provider near the app root (see `use-gateway.tsx`) —
 * never instantiated per screen, so presence/typing traffic and read-state
 * writes are not multiplied by however many tabs or components are mounted.
 */
export class GatewayClient {
	private socket: WebSocketLike | null = null
	private state: GatewayConnectionState = 'idle'
	private attempt = 0
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null
	private explicitlyClosed = false
	private userId: string | null = null
	private readonly stateHandlers = new Set<StateHandler>()
	private readonly eventHandlers = new Map<
		GatewayEventType,
		Set<EventHandler>
	>()

	constructor(private readonly options: GatewayClientOptions) {}

	getState(): GatewayConnectionState {
		return this.state
	}

	/** The `user_id` the server resolved at identify time — `null` until the
	 * first `ready`. Used to tell "my own message" from anyone else's,
	 * notably to reconcile an optimistic send against its own gateway echo. */
	getUserId(): string | null {
		return this.userId
	}

	connect(): void {
		this.explicitlyClosed = false
		if (
			this.state === 'connecting' ||
			this.state === 'open' ||
			this.state === 'reconnecting'
		) {
			return
		}

		const token = this.options.getToken()
		if (!token) {
			this.setState('idle')
			return
		}

		this.setState('connecting')
		this.openSocket(token)
	}

	disconnect(): void {
		// Nothing was ever attempted — stay `idle` rather than claiming a
		// connection was closed. Lets a provider call `disconnect()`
		// defensively on mount without it reading as a real state change.
		if (this.state === 'idle') return

		this.explicitlyClosed = true
		if (this.reconnectTimer !== null) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		this.socket?.close()
		this.setState('closed')
	}

	on<T extends GatewayEventType>(
		type: T,
		handler: (event: GatewayEvent<T>) => void,
	): () => void {
		const handlers = this.eventHandlers.get(type) ?? new Set<EventHandler>()
		handlers.add(handler as EventHandler)
		this.eventHandlers.set(type, handlers)
		return () => {
			handlers.delete(handler as EventHandler)
		}
	}

	onStateChange(handler: StateHandler): () => void {
		this.stateHandlers.add(handler)
		return () => {
			this.stateHandlers.delete(handler)
		}
	}

	private openSocket(token: string): void {
		const factory =
			this.options.wsFactory ??
			((url: string) => new WebSocket(url) as unknown as WebSocketLike)
		const socket = factory(this.options.url)
		this.socket = socket

		socket.onopen = () => {
			this.send({ op: 'identify', token })
		}
		socket.onmessage = (event) => {
			this.handleMessage(event.data)
		}
		socket.onclose = () => {
			this.handleClose()
		}
		socket.onerror = () => {
			// The close handler runs right after in every browser implementation;
			// nothing actionable here beyond letting that reconnect logic fire.
		}
	}

	private send(message: ClientMessage): void {
		this.socket?.send(JSON.stringify(message))
	}

	private handleMessage(raw: string): void {
		let message: ServerMessage
		try {
			message = JSON.parse(raw) as ServerMessage
		} catch {
			return
		}

		switch (message.op) {
			case 'ready': {
				this.attempt = 0
				this.userId = message.user_id
				this.setState('open')
				return
			}
			case 'heartbeat': {
				this.send({ op: 'heartbeat_ack' })
				return
			}
			case 'dispatch': {
				if (!isGatewayEventType(message.t)) return
				const handlers = this.eventHandlers.get(message.t)
				if (!handlers || handlers.size === 0) return
				const event = { type: message.t, data: message.d } as GatewayEvent
				for (const handler of handlers) handler(event)
				return
			}
			case 'close': {
				// The server closes the socket right after this frame; `onclose`
				// drives the reconnect below. Nothing else to do with the reason
				// today beyond what a future "why did we disconnect" surface uses.
				return
			}
		}
	}

	private handleClose(): void {
		this.socket = null
		if (this.explicitlyClosed) {
			this.setState('closed')
			return
		}

		this.setState('reconnecting')
		const delay = computeBackoffMs(this.attempt, this.options.backoff)
		this.attempt += 1
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null
			const token = this.options.getToken()
			if (!token) {
				this.setState('idle')
				return
			}
			this.setState('connecting')
			this.openSocket(token)
		}, delay)
	}

	private setState(next: GatewayConnectionState): void {
		if (this.state === next) return
		this.state = next
		for (const handler of this.stateHandlers) handler(next)
	}
}
