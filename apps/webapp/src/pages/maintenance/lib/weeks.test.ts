import { describe, expect, it } from 'vitest'
import {
	dateInIsoWeek,
	dayOfMonth,
	isoWeekOf,
	isoWeeksInYear,
	mondayOfIsoWeek,
	weekRange,
} from '#/pages/maintenance/lib/weeks'

describe('isoWeekOf', () => {
	it('puts the days of a week in the same week, Monday to Sunday', () => {
		expect(isoWeekOf('2026-10-19')).toEqual({ year: 2026, week: 43 })
		expect(isoWeekOf('2026-10-21')).toEqual({ year: 2026, week: 43 })
		expect(isoWeekOf('2026-10-25')).toEqual({ year: 2026, week: 43 })
		expect(isoWeekOf('2026-10-26')).toEqual({ year: 2026, week: 44 })
	})

	it('belongs to the year of the Thursday at the turn of a year', () => {
		expect(isoWeekOf('2026-01-01')).toEqual({ year: 2026, week: 1 })
		expect(isoWeekOf('2027-01-01')).toEqual({ year: 2026, week: 53 })
		expect(isoWeekOf('2024-12-30')).toEqual({ year: 2025, week: 1 })
	})
})

describe('isoWeeksInYear', () => {
	it('knows which years have a week 53', () => {
		expect(isoWeeksInYear(2026)).toBe(53)
		expect(isoWeeksInYear(2027)).toBe(52)
		expect(isoWeeksInYear(2025)).toBe(52)
	})
})

describe('mondayOfIsoWeek', () => {
	it('gives the Monday, across a month and a year boundary', () => {
		expect(mondayOfIsoWeek(2026, 43)).toBe('2026-10-19')
		expect(mondayOfIsoWeek(2026, 44)).toBe('2026-10-26')
		expect(mondayOfIsoWeek(2026, 1)).toBe('2025-12-29')
	})
})

describe('dateInIsoWeek', () => {
	it('finds the day of the month inside the week', () => {
		expect(dateInIsoWeek(2026, 43, 21)).toBe('2026-10-21')
	})

	it('picks the right month when the week straddles two', () => {
		// S44 runs from Monday 26 October to Sunday 1 November.
		expect(dateInIsoWeek(2026, 44, 28)).toBe('2026-10-28')
		expect(dateInIsoWeek(2026, 44, 1)).toBe('2026-11-01')
	})

	it('refuses a day the week does not contain', () => {
		expect(dateInIsoWeek(2026, 43, 5)).toBeNull()
		expect(dateInIsoWeek(2026, 43, 0)).toBeNull()
		expect(dateInIsoWeek(2026, 43, 32)).toBeNull()
	})
})

describe('small helpers', () => {
	it('reads the day of the month', () => {
		expect(dayOfMonth('2026-10-07')).toBe(7)
	})

	it('lists a week range inclusively', () => {
		expect(weekRange(12, 15)).toEqual([12, 13, 14, 15])
		expect(weekRange(15, 12)).toEqual([])
	})
})
