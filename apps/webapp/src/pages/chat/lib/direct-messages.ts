import type { PresenceStatus } from '#/hooks/use-presence'
import { userHue } from '#/pages/chat/lib/format'

export interface DirectMessageItem {
	channelId: string
	name: string
	hue: number
	presence: PresenceStatus
	unreadCount: number
}

interface DirectChannelLike {
	channel_id: string
	peer_user_id: string
}

interface DirectoryEntryLike {
	user_id: string
	display_name: string
}

const UNKNOWN_PEER = 'Ancien membre'

/** The sidebar rows for the caller's direct messages, people with something
 * unread first, then alphabetical. A peer who left the organization keeps
 * their conversation, named as a former member. */
export function buildDirectMessageItems(
	directs: readonly DirectChannelLike[],
	directory: readonly DirectoryEntryLike[],
	presenceByUser: ReadonlyMap<string, PresenceStatus>,
	unreadCounts: ReadonlyMap<string, number>,
): DirectMessageItem[] {
	const nameByUser = new Map(
		directory.map((entry) => [entry.user_id, entry.display_name]),
	)
	return directs
		.map((direct) => ({
			channelId: direct.channel_id,
			name: nameByUser.get(direct.peer_user_id) ?? UNKNOWN_PEER,
			hue: userHue(direct.peer_user_id),
			presence: presenceByUser.get(direct.peer_user_id) ?? 'OFFLINE',
			unreadCount: unreadCounts.get(direct.channel_id) ?? 0,
		}))
		.sort(
			(a, b) =>
				Number(b.unreadCount > 0) - Number(a.unreadCount > 0) ||
				a.name.localeCompare(b.name, 'fr'),
		)
}
