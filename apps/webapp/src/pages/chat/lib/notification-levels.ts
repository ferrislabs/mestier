import { Bell, BellOff, type LucideIcon, MessageSquare } from 'lucide-react'
import type { NotificationLevel } from '#/hooks/use-chat'

export interface NotificationLevelOption {
	value: NotificationLevel
	label: string
	icon: LucideIcon
}

export const NOTIFICATION_LEVELS: readonly NotificationLevelOption[] = [
	{ value: 'ALL', label: 'Tous les messages', icon: MessageSquare },
	{ value: 'MENTIONS', label: 'Mentions uniquement', icon: Bell },
	{ value: 'NOTHING', label: 'Aucune notification', icon: BellOff },
]

/** What applies when the person never chose: a channel only tells about
 * mentions (a direct message, which is meant for them, tells about everything
 * and is not set from the channel list). */
export const DEFAULT_CHANNEL_LEVEL: NotificationLevel = 'MENTIONS'

export function effectiveNotificationLevel(
	chosen: ReadonlyMap<string, NotificationLevel>,
	channelId: string,
): NotificationLevel {
	return chosen.get(channelId) ?? DEFAULT_CHANNEL_LEVEL
}
