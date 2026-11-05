import { describe, expect, it } from 'vitest'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'
import {
	buildTimeline,
	filterMessagesByQuery,
	formatDayLabel,
} from '#/pages/chat/lib/timeline'

function entry(
	id: string,
	author: string,
	createdAt: string,
	content = 'texte',
	filenames: string[] = [],
): DisplayMessage {
	return {
		status: 'sent',
		message: {
			id,
			author_user_id: author,
			author_type: 'USER',
			created_at: createdAt,
			content,
			attachments: filenames.map((filename) => ({
				filename,
				mime_type: 'application/pdf',
				size_bytes: 1,
				storage_key: filename,
			})),
		},
	} as unknown as DisplayMessage
}

describe('formatDayLabel', () => {
	const now = new Date(2026, 10, 5, 12)

	it('names today and yesterday', () => {
		expect(formatDayLabel(new Date(2026, 10, 5, 8), now)).toBe('Aujourd’hui')
		expect(formatDayLabel(new Date(2026, 10, 4, 8), now)).toBe('Hier')
	})

	it('spells older days out with a capital', () => {
		expect(formatDayLabel(new Date(2026, 10, 2, 8), now)).toBe(
			'Lundi 2 novembre',
		)
	})
})

describe('buildTimeline', () => {
	it('opens each day with a separator', () => {
		const timeline = buildTimeline([
			entry('a', 'u1', new Date(2026, 9, 1, 9).toISOString()),
			entry('b', 'u1', new Date(2026, 9, 2, 9).toISOString()),
		])

		expect(timeline.map((item) => item.kind)).toEqual([
			'day',
			'message',
			'day',
			'message',
		])
	})

	it('groups consecutive messages of one author within five minutes', () => {
		const timeline = buildTimeline([
			entry('a', 'u1', new Date(2026, 9, 1, 9, 0).toISOString()),
			entry('b', 'u1', new Date(2026, 9, 1, 9, 3).toISOString()),
			entry('c', 'u1', new Date(2026, 9, 1, 9, 10).toISOString()),
			entry('d', 'u2', new Date(2026, 9, 1, 9, 11).toISOString()),
		])

		const flags = timeline.flatMap((item) =>
			item.kind === 'message' ? [item.continuation] : [],
		)
		expect(flags).toEqual([false, true, false, false])
	})

	it('is empty without messages', () => {
		expect(buildTimeline([])).toEqual([])
	})
})

describe('filterMessagesByQuery', () => {
	const messages = [
		entry('a', 'u1', '2026-10-01T09:00:00Z', 'Devis Dupont validé'),
		entry('b', 'u1', '2026-10-01T09:01:00Z', 'photo', ['Façade-nord.jpg']),
	]

	it('keeps everything for a blank query', () => {
		expect(filterMessagesByQuery(messages, '  ')).toHaveLength(2)
	})

	it('matches content without regard to case or accents', () => {
		expect(
			filterMessagesByQuery(messages, 'VALIDE').map((m) => m.message.id),
		).toEqual(['a'])
	})

	it('matches attachment file names', () => {
		expect(
			filterMessagesByQuery(messages, 'facade').map((m) => m.message.id),
		).toEqual(['b'])
	})
})
