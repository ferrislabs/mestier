import type { PresenceStatus } from '#/hooks/use-presence'

export interface MemberItem {
	id: string
	name: string
	hue: number
	presence: PresenceStatus
}
