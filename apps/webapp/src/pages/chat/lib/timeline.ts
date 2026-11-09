import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'

export const GROUPING_WINDOW_MS = 5 * 60 * 1000

const MS_PER_DAY = 24 * 60 * 60 * 1000

const fullDayFormatter = new Intl.DateTimeFormat('fr-FR', {
	weekday: 'long',
	day: 'numeric',
	month: 'long',
})

export interface UnreadMarker {
	messageId: string
	count: number
}

export type TimelineEntry =
	| { kind: 'day'; key: string; label: string }
	| { kind: 'unread'; key: string; count: number }
	| {
			kind: 'message'
			key: string
			entry: DisplayMessage
			continuation: boolean
	  }

function startOfDay(date: Date): number {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()
}

function daysBetween(date: Date, now: Date): number {
	return Math.round((startOfDay(now) - startOfDay(date)) / MS_PER_DAY)
}

function capitalize(text: string): string {
	return text.charAt(0).toUpperCase() + text.slice(1)
}

function normalizeForSearch(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
}

export function formatDayLabel(date: Date, now: Date): string {
	const difference = daysBetween(date, now)
	if (difference === 0) return 'Aujourd’hui'
	if (difference === 1) return 'Hier'
	return capitalize(fullDayFormatter.format(date))
}

function isContinuation(previous: DisplayMessage, current: DisplayMessage) {
	const elapsed =
		new Date(current.message.created_at).getTime() -
		new Date(previous.message.created_at).getTime()
	return (
		previous.message.author_user_id === current.message.author_user_id &&
		previous.message.author_type === current.message.author_type &&
		elapsed >= 0 &&
		elapsed <= GROUPING_WINDOW_MS
	)
}

export function entryKey(entry: DisplayMessage): string {
	return entry.tempId ?? entry.message.id
}

export function buildTimeline(
	messages: readonly DisplayMessage[],
	unread?: UnreadMarker,
): TimelineEntry[] {
	const entries: TimelineEntry[] = []
	let previous: DisplayMessage | undefined

	for (const entry of messages) {
		const sentAt = new Date(entry.message.created_at)
		const startsNewDay =
			previous === undefined ||
			startOfDay(new Date(previous.message.created_at)) !== startOfDay(sentAt)

		if (startsNewDay) {
			entries.push({
				kind: 'day',
				key: `day-${startOfDay(sentAt)}`,
				label: formatDayLabel(sentAt, new Date()),
			})
		}
		const startsUnread =
			unread !== undefined && entry.message.id === unread.messageId
		if (startsUnread) {
			entries.push({
				kind: 'unread',
				key: `unread-${unread.messageId}`,
				count: unread.count,
			})
		}
		entries.push({
			kind: 'message',
			key: entryKey(entry),
			entry,
			continuation:
				!startsNewDay &&
				!startsUnread &&
				previous !== undefined &&
				isContinuation(previous, entry),
		})
		previous = entry
	}

	return entries
}

export function filterMessagesByQuery(
	messages: readonly DisplayMessage[],
	query: string,
): DisplayMessage[] {
	const needle = normalizeForSearch(query.trim())
	if (needle === '') return [...messages]

	return messages.filter(({ message }) =>
		normalizeForSearch(
			`${message.content} ${message.attachments.map((attachment) => attachment.filename).join(' ')}`,
		).includes(needle),
	)
}
