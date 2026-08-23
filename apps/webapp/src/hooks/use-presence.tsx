import { useMutation } from '@tanstack/react-query'
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
} from 'react'
import type { Schemas } from '#/api/api.client'
import { useGatewayConnectionState, useGatewayEvent } from '#/hooks/use-gateway'

const ORG_PRESENCE_PATH =
	'/api/v1/chat/organizations/{organization_id}/presence'

export type PresenceStatus = Schemas.PresenceStatus

interface PresenceContextValue {
	statuses: ReadonlyMap<string, PresenceStatus>
}

const PresenceContext = createContext<PresenceContextValue | null>(null)

function useSetPresenceMutation() {
	return useMutation({
		...window.tanstackApi.mutation('put', ORG_PRESENCE_PATH).mutationOptions,
	})
}

/**
 * Tracks presence for the active organization: reads other members' status
 * off the gateway, and publishes the caller's own status on the three
 * moments the issue calls out — connect, visibility change, and disconnect.
 *
 * There is no REST snapshot of "who is online right now" (`presence/` only
 * exposes `PUT .../presence`, not a list — `list_presence` exists on the
 * core use case but was never wired to a route). So this map only knows
 * about a member once a `PRESENCE_UPDATE` has crossed the wire since this
 * tab connected; there is no cold-start snapshot. Flagged rather than
 * worked around — adding the route is backend work, out of scope here.
 *
 * Ephemeral by design: plain component state, never a query cache entry,
 * gone on reload. A `PRESENCE_UPDATE` naturally repopulates it.
 */
export function PresenceProvider({
	organizationId,
	children,
}: {
	organizationId: string
	children: React.ReactNode
}) {
	const [statuses, setStatuses] = useState<Map<string, PresenceStatus>>(
		new Map(),
	)
	const connectionState = useGatewayConnectionState()
	const setPresence = useSetPresenceMutation()
	const setPresenceRef = useRef(setPresence.mutate)
	setPresenceRef.current = setPresence.mutate

	useGatewayEvent('PRESENCE_UPDATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		setStatuses((previous) => {
			const next = new Map(previous)
			next.set(event.data.user_id, event.data.status)
			return next
		})
	})

	const publish = useCallback(
		(status: PresenceStatus) => {
			setPresenceRef.current({
				path: { organization_id: organizationId },
				body: { status },
			})
		},
		[organizationId],
	)

	// Connect: the moment the socket is ready, this org's presence goes live.
	useEffect(() => {
		if (connectionState === 'open') publish('ONLINE')
	}, [connectionState, publish])

	// Visibility: a hidden tab is not "reachable right now" and a small-team
	// chat has no "away" status to fall back to, so it reads as OFFLINE
	// rather than staying ONLINE while nobody is looking. Republishes ONLINE
	// the moment the tab is visible again.
	useEffect(() => {
		function handleVisibilityChange() {
			if (connectionState !== 'open') return
			publish(document.hidden ? 'OFFLINE' : 'ONLINE')
		}
		document.addEventListener('visibilitychange', handleVisibilityChange)
		return () =>
			document.removeEventListener('visibilitychange', handleVisibilityChange)
	}, [connectionState, publish])

	// Disconnect: the server does not infer OFFLINE from the socket closing
	// (self-declared by design, see `gateway.rs`) — silence here is exactly
	// how someone stays green after they left.
	useEffect(() => {
		return () => publish('OFFLINE')
	}, [publish])

	return (
		<PresenceContext.Provider value={{ statuses }}>
			{children}
		</PresenceContext.Provider>
	)
}

/** A member's known status, or `'UNKNOWN'` when no event has told us yet. */
export function usePresenceStatus(
	userId: string | undefined,
): PresenceStatus | 'UNKNOWN' {
	const context = useContext(PresenceContext)
	if (!context || !userId) return 'UNKNOWN'
	return context.statuses.get(userId) ?? 'UNKNOWN'
}

/** How many known members are currently `ONLINE`. */
export function useOnlineCount(): number {
	const context = useContext(PresenceContext)
	if (!context) return 0
	let count = 0
	for (const status of context.statuses.values()) {
		if (status === 'ONLINE') count += 1
	}
	return count
}
