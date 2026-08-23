import { useStore } from '@tanstack/react-store'
import {
	createContext,
	useContext,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react'
import {
	GatewayClient,
	type GatewayConnectionState,
} from '#/lib/gateway/gateway-client'
import { resolveGatewayUrl } from '#/lib/gateway/gateway-url'
import type { GatewayEvent, GatewayEventType } from '#/lib/gateway/wire-types'
import { authStore } from '#/store/auth.store'

interface GatewayContextValue {
	client: GatewayClient
}

const GatewayContext = createContext<GatewayContextValue | null>(null)

/**
 * Mounts the single gateway connection for the session. Belongs near the
 * app root (see `routes/_app.tsx`) — one instance regardless of how many
 * organizations or chat screens render underneath it, so presence/typing
 * traffic and read-state writes are not multiplied per tab.
 *
 * Connects once the OIDC session is authenticated and disconnects when it
 * is not (sign-out, expiry with no recovery) — screens never call `connect`
 * or `disconnect` themselves.
 */
export function GatewayProvider({ children }: { children: React.ReactNode }) {
	const isAuthenticated = useStore(authStore, (s) => s.isAuthenticated)

	const client = useMemo(() => {
		const apiUrl = typeof window !== 'undefined' ? window.apiUrl : undefined
		const pageOrigin =
			typeof window !== 'undefined' ? window.location.origin : ''
		return new GatewayClient({
			url: resolveGatewayUrl(apiUrl, pageOrigin),
			getToken: () => authStore.state.accessToken,
		})
	}, [])

	useEffect(() => {
		if (isAuthenticated) {
			client.connect()
		} else {
			client.disconnect()
		}
	}, [client, isAuthenticated])

	useEffect(() => {
		return () => client.disconnect()
	}, [client])

	return (
		<GatewayContext.Provider value={{ client }}>
			{children}
		</GatewayContext.Provider>
	)
}

function useGatewayClient(): GatewayClient {
	const context = useContext(GatewayContext)
	if (!context) {
		throw new Error('useGateway hooks must be used within a GatewayProvider')
	}
	return context.client
}

/** The connection's current lifecycle state — drives a "reconnecting…"
 * indicator rather than letting the chat silently stop receiving. */
export function useGatewayConnectionState(): GatewayConnectionState {
	const client = useGatewayClient()
	const [state, setState] = useState(client.getState())

	useEffect(() => client.onStateChange(setState), [client])

	return state
}

/** The caller's own `user_id`, once known (after the first `ready`) — `null`
 * before that. Used to tell "my own message" from anyone else's. */
export function useGatewayUserId(): string | null {
	const client = useGatewayClient()
	const [userId, setUserId] = useState(client.getUserId())

	useEffect(
		() => client.onStateChange(() => setUserId(client.getUserId())),
		[client],
	)

	return userId
}

/**
 * Subscribes to one gateway event type for the lifetime of the calling
 * component. The handler is kept in a ref so callers can pass an inline
 * closure without re-subscribing on every render.
 */
export function useGatewayEvent<T extends GatewayEventType>(
	type: T,
	handler: (event: GatewayEvent<T>) => void,
): void {
	const client = useGatewayClient()
	const handlerRef = useRef(handler)
	handlerRef.current = handler

	useEffect(
		() => client.on(type, (event) => handlerRef.current(event)),
		[client, type],
	)
}
