import { describe, expect, it } from 'vitest'
import { addDaysIso, isIsoDate } from './iso-date'

describe('isIsoDate', () => {
	it('accepts real calendar days only', () => {
		expect(isIsoDate('2026-06-10')).toBe(true)
		expect(isIsoDate('2026-13-40')).toBe(false)
		expect(isIsoDate('2026-02-30')).toBe(false)
		expect(isIsoDate('')).toBe(false)
		expect(isIsoDate('10/06/2026')).toBe(false)
	})
})

describe('addDaysIso', () => {
	it('adds calendar days across month and year ends', () => {
		expect(addDaysIso('2026-06-10', 30)).toBe('2026-07-10')
		expect(addDaysIso('2026-12-20', 15)).toBe('2027-01-04')
		expect(addDaysIso('2026-06-10', 0)).toBe('2026-06-10')
	})
})
