import { describe, expect, it } from 'vitest'
import { sumCounts, tutorTab } from '#/pages/cursus/tutors/lib/tutors'

describe('tutors lib', () => {
	it('sums the counts once all are known', () => {
		expect(sumCounts(7, 3)).toBe(10)
		expect(sumCounts(7, undefined)).toBeUndefined()
	})

	it('reads the tab from the search', () => {
		expect(tutorTab('ecole')).toBe('ecole')
		expect(tutorTab(undefined)).toBe('entreprise')
	})
})
