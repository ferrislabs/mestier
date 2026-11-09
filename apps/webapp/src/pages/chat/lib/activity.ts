import type { ActivityEntry } from '#/hooks/use-chat'
import { type AuthorDirectory, resolveAuthor } from '#/pages/chat/lib/authors'
import { formatTime } from '#/pages/chat/lib/format'
import { formatDayLabel } from '#/pages/chat/lib/timeline'

export type ActivityFilter = 'all' | 'mentions' | 'unread'

/** One line of the activity page, ready to draw. */
export interface ActivityItem {
	key: string
	channelId: string
	kind: 'MENTION' | 'REPLY' | 'UNREAD'
	authorName: string
	authorHue: number
	/** `#canal`, or the other person's name for a direct message. */
	where: string
	excerpt: string
	when: string
	read: boolean
	unreadCount: number | null
	notificationId: string | null
}

const MENTION_TOKEN = /<(@&|@|#)([0-9a-f-]{36})>/gi

/** Replaces the mention tokens the server stores (`<@id>`) by readable text. */
export function humanizeMentions(
	text: string,
	authors: AuthorDirectory,
	channelNames: ReadonlyMap<string, string>,
): string {
	return text.replace(MENTION_TOKEN, (_token, marker: string, id: string) => {
		if (marker === '@') return `@${resolveAuthor(authors, id).name}`
		if (marker === '#') return `#${channelNames.get(id) ?? 'canal'}`
		return '@rôle'
	})
}

export function formatActivityTime(date: Date, now: Date): string {
	const day = formatDayLabel(date, now)
	return day === 'Aujourd’hui'
		? formatTime(date)
		: `${day} · ${formatTime(date)}`
}

export function buildActivityItems(
	entries: readonly ActivityEntry[],
	authors: AuthorDirectory,
	channelNames: ReadonlyMap<string, string>,
	directNames: ReadonlyMap<string, string>,
	now: Date,
): ActivityItem[] {
	return entries.map((entry) => {
		const author = resolveAuthor(authors, entry.author_user_id)
		const channelName = channelNames.get(entry.channel_id)
		const directName = directNames.get(entry.channel_id)
		return {
			key: entry.notification_id ?? `unread:${entry.channel_id}`,
			channelId: entry.channel_id,
			kind: entry.kind as ActivityItem['kind'],
			authorName: author.name,
			authorHue: author.hue,
			where: directName ?? (channelName ? `#${channelName}` : 'Canal'),
			excerpt: humanizeMentions(entry.excerpt, authors, channelNames),
			when: formatActivityTime(new Date(entry.created_at), now),
			read: entry.read,
			unreadCount: entry.unread_count ?? null,
			notificationId: entry.notification_id ?? null,
		}
	})
}

export function filterActivity(
	items: readonly ActivityItem[],
	filter: ActivityFilter,
): ActivityItem[] {
	if (filter === 'mentions') {
		return items.filter((item) => item.kind !== 'UNREAD')
	}
	if (filter === 'unread') {
		return items.filter((item) => item.kind === 'UNREAD')
	}
	return [...items]
}

/** Mentions nobody has read yet: what the sidebar badge counts. */
export function countUnreadMentions(items: readonly ActivityItem[]): number {
	return items.filter((item) => item.kind !== 'UNREAD' && !item.read).length
}
