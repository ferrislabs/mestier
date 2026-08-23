/**
 * Hand-written mirror of the gateway wire protocol implemented in
 * `libs/handlers-discord/src/gateway.rs` (`ClientMessage` / `ServerMessage`)
 * and `libs/core/src/infrastructure/realtime/wire.rs` (`GatewayEvent`).
 *
 * There is no TS-generation pipeline for these types today (the OpenAPI
 * client only covers REST routes, and the gateway is a bare WebSocket, not
 * documented in the OpenAPI schema). Deriving them would need a backend
 * change (e.g. wiring `ts-rs` on the Rust side) — noted in issue #323 rather
 * than added here, since this workstream touches no `.rs` files. Keep this
 * file in lockstep with the Rust source by hand until that exists.
 */
import type { Schemas } from '#/api/api.client'

// ── Client → Server ────────────────────────────────────────────────────────

export type ClientMessage =
	| { op: 'identify'; token: string }
	| { op: 'heartbeat_ack' }

// ── Server → Client ────────────────────────────────────────────────────────

export type ServerMessage =
	| { op: 'ready'; user_id: string }
	| { op: 'dispatch'; t: GatewayEventType; d: unknown }
	| { op: 'heartbeat' }
	| { op: 'close'; reason: string }

// ── Dispatch payloads ──────────────────────────────────────────────────────

export type GatewayEventType = keyof GatewayEventDataMap

/**
 * One entry per `GatewayEvent` variant in `wire.rs`. Domain structs
 * (`Message`, `Category`, `Channel`, `Presence`) serialize with the same
 * field names as their REST `*Response` counterparts (`response.rs` maps
 * them 1:1), so the REST `Schemas.*Response` types are reused here rather
 * than duplicated. `NOTIFICATION_CREATE` is the one exception: the gateway
 * carries the raw `Notification` domain struct (with `organization_id` and
 * `user_id`), while the REST `NotificationResponse` deliberately omits both.
 */
export interface GatewayEventDataMap {
	MESSAGE_CREATE: Schemas.MessageResponse
	MESSAGE_UPDATE: Schemas.MessageResponse
	MESSAGE_DELETE: {
		organization_id: string
		channel_id: string
		message_id: string
	}
	REACTION_ADD: {
		organization_id: string
		channel_id: string
		message_id: string
		emoji: string
		user_id: string
	}
	REACTION_REMOVE: {
		organization_id: string
		channel_id: string
		message_id: string
		emoji: string
		user_id: string
	}
	CATEGORY_CREATE: Schemas.CategoryResponse
	CATEGORY_UPDATE: Schemas.CategoryResponse
	CATEGORY_DELETE: { organization_id: string; category_id: string }
	CHANNEL_CREATE: Schemas.ChannelResponse
	CHANNEL_UPDATE: Schemas.ChannelResponse
	CHANNEL_DELETE: { organization_id: string; channel_id: string }
	THREAD_CREATE: Schemas.ChannelResponse
	THREAD_UPDATE: Schemas.ChannelResponse
	THREAD_DELETE: { organization_id: string; channel_id: string }
	PRESENCE_UPDATE: Schemas.PresenceResponse
	TYPING_START: {
		organization_id: string
		channel_id: string
		user_id: string
		ttl_ms: number
	}
	CHANNEL_READ: {
		organization_id: string
		channel_id: string
		user_id: string
		last_read_message_id: string | null
	}
	NOTIFICATION_CREATE: {
		id: string
		organization_id: string
		user_id: string
		channel_id: string
		message_id: string
		kind: string
		read_at: string | null
		created_at: string
	}
}

export type GatewayEvent<T extends GatewayEventType = GatewayEventType> = {
	[K in T]: { type: K; data: GatewayEventDataMap[K] }
}[T]

export function isGatewayEventType(value: string): value is GatewayEventType {
	return (GATEWAY_EVENT_TYPES as readonly string[]).includes(value)
}

const GATEWAY_EVENT_TYPES = [
	'MESSAGE_CREATE',
	'MESSAGE_UPDATE',
	'MESSAGE_DELETE',
	'REACTION_ADD',
	'REACTION_REMOVE',
	'CATEGORY_CREATE',
	'CATEGORY_UPDATE',
	'CATEGORY_DELETE',
	'CHANNEL_CREATE',
	'CHANNEL_UPDATE',
	'CHANNEL_DELETE',
	'THREAD_CREATE',
	'THREAD_UPDATE',
	'THREAD_DELETE',
	'PRESENCE_UPDATE',
	'TYPING_START',
	'CHANNEL_READ',
	'NOTIFICATION_CREATE',
] as const satisfies readonly GatewayEventType[]
