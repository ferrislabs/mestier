import { describe, expect, it } from 'vitest'
import {
	formatRelative,
	initials,
	joinNames,
	memberHue,
} from '#/pages/support/lib/format'

const NOW = new Date(2026, 10, 5, 14, 30)

describe('formatRelative', () => {
	it('says "à l’instant" under a minute', () => {
		expect(
			formatRelative(new Date(2026, 10, 5, 14, 29, 40).toISOString(), NOW),
		).toBe('à l’instant')
	})

	it('counts minutes then hours within the same day', () => {
		expect(
			formatRelative(new Date(2026, 10, 5, 14, 10).toISOString(), NOW),
		).toBe('il y a 20 min')
		expect(formatRelative(new Date(2026, 10, 5, 9, 0).toISOString(), NOW)).toBe(
			'il y a 5 h',
		)
	})

	it('names yesterday with the time and older days with the date', () => {
		expect(
			formatRelative(new Date(2026, 10, 4, 9, 5).toISOString(), NOW),
		).toMatch(/^hier à 09:05$/)
		expect(
			formatRelative(new Date(2026, 9, 20, 9, 5).toISOString(), NOW),
		).toMatch(/20 oct/)
	})
})

describe('initials and joinNames', () => {
	it('keeps the first two initials', () => {
		expect(initials('Baptiste Jean Parmantier')).toBe('BJ')
		expect(initials('')).toBe('')
	})

	it('joins names the French way', () => {
		expect(joinNames([])).toBe('')
		expect(joinNames(['Lucie'])).toBe('Lucie')
		expect(joinNames(['Lucie', 'Karim', 'Julien'])).toBe(
			'Lucie, Karim et Julien',
		)
	})
})

describe('memberHue', () => {
	it('is stable and within the hue circle', () => {
		expect(memberHue('member-1')).toBe(memberHue('member-1'))
		expect(memberHue('member-1')).toBeGreaterThanOrEqual(0)
		expect(memberHue('member-1')).toBeLessThan(360)
	})
})
