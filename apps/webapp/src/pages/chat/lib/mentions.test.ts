import { describe, expect, it } from 'vitest'
import { splitMentions } from '#/pages/chat/lib/mentions'

describe('splitMentions', () => {
	it('returns a single plain segment without mention', () => {
		expect(splitMentions('bonjour tout le monde')).toEqual([
			{ key: '0', text: 'bonjour tout le monde', isMention: false },
		])
	})

	it('isolates a first and last name mention', () => {
		const segments = splitMentions('salut @Karim Benali, ok ?')

		expect(segments.map((segment) => segment.text)).toEqual([
			'salut ',
			'@Karim Benali',
			', ok ?',
		])
		expect(segments.map((segment) => segment.isMention)).toEqual([
			false,
			true,
			false,
		])
	})

	it('keeps accented names whole', () => {
		const mentions = splitMentions('merci @Élodie').filter(
			(segment) => segment.isMention,
		)

		expect(mentions.map((segment) => segment.text)).toEqual(['@Élodie'])
	})

	it('ignores a lowercase handle', () => {
		expect(splitMentions('écris à @karim').some((s) => s.isMention)).toBe(false)
	})

	it('gives every segment a distinct key', () => {
		const keys = splitMentions('@Karim et @Lucie').map((segment) => segment.key)

		expect(new Set(keys).size).toBe(keys.length)
	})
})
