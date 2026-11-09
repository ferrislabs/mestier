import type { PresenceStatus } from '#/hooks/use-presence'

/** The backend's `DND` reads as "Absent" in the rework's vocabulary. */
export const PRESENCE_LABELS: Record<PresenceStatus, string> = {
	ONLINE: 'En ligne',
	DND: 'Absent',
	OFFLINE: 'Hors ligne',
}

/** Groups of the members dialog, most reachable first. */
export const PRESENCE_ORDER: readonly PresenceStatus[] = [
	'ONLINE',
	'DND',
	'OFFLINE',
]
