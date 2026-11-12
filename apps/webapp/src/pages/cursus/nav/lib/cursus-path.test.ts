import { describe, expect, it } from 'vitest'
import { activeCursusEntry } from '#/pages/cursus/nav/lib/cursus-path'

describe('activeCursusEntry', () => {
	it('matches a section root', () => {
		expect(activeCursusEntry('/cursus/promotions')).toBe('promotions')
		expect(activeCursusEntry('/cursus/trinomes')).toBe('triads')
	})

	it('keeps the section active on a detail page', () => {
		expect(activeCursusEntry('/cursus/entreprises/abc')).toBe('companies')
	})

	it('does not confuse my triads with triads', () => {
		expect(activeCursusEntry('/cursus/mes-trinomes/x')).toBe('myTriads')
	})

	it('returns null outside any section', () => {
		expect(activeCursusEntry('/cursus')).toBeNull()
	})
})
