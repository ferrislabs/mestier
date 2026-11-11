import { describe, expect, it } from 'vitest'
import {
	filterMentionOptions,
	type MentionOption,
} from '#/pages/knowledge-documents/lib/mention-options'

const options: MentionOption[] = [
	{ kind: 'customer', id: 'c1', label: 'Jardins du Parc' },
	{ kind: 'customer', id: 'c2', label: 'Élodie Martin' },
	{ kind: 'project', id: 'p1', label: 'Parc de la Mairie' },
	{ kind: 'quote', id: 'q1', label: 'DEV-2026-014' },
]

describe('filterMentionOptions', () => {
	it('lists the first options when nothing is typed', () => {
		expect(filterMentionOptions(options, '', 2)).toHaveLength(2)
	})

	it('ignores case and accents', () => {
		expect(filterMentionOptions(options, 'elodie').map((o) => o.id)).toEqual([
			'c2',
		])
	})

	it('puts the labels that start with the query first', () => {
		expect(filterMentionOptions(options, 'parc').map((o) => o.id)).toEqual([
			'p1',
			'c1',
		])
	})

	it('finds a quote by its reference', () => {
		expect(filterMentionOptions(options, '2026-014').map((o) => o.id)).toEqual([
			'q1',
		])
	})

	it('returns nothing when nothing matches', () => {
		expect(filterMentionOptions(options, 'zzz')).toEqual([])
	})
})
