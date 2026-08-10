import { describe, expect, it } from 'vitest'
import {
	computeWindow,
	enumerateDays,
	shiftDate,
} from '#/pages/planning/lib/window'

describe('computeWindow', () => {
	it('day view: the window is the day itself', () => {
		expect(computeWindow('day', '2026-08-07')).toEqual({
			from: '2026-08-07',
			to: '2026-08-07',
		})
	})

	it('week view: Monday to Sunday containing the date', () => {
		// 2026-08-07 is a Friday.
		expect(computeWindow('week', '2026-08-07')).toEqual({
			from: '2026-08-03',
			to: '2026-08-09',
		})
	})

	it('week view: crosses months when the week straddles two of them', () => {
		// 2026-08-31 is a Monday; the week ends in September.
		expect(computeWindow('week', '2026-08-31')).toEqual({
			from: '2026-08-31',
			to: '2026-09-06',
		})
	})

	it('week view: crosses years when the week straddles two of them', () => {
		// 2025-12-29 is a Monday; the week ends in January 2026.
		expect(computeWindow('week', '2025-12-29')).toEqual({
			from: '2025-12-29',
			to: '2026-01-04',
		})
		expect(computeWindow('week', '2026-01-01')).toEqual({
			from: '2025-12-29',
			to: '2026-01-04',
		})
	})

	it('month view: first to last day of the month containing the date', () => {
		expect(computeWindow('month', '2026-02-15')).toEqual({
			from: '2026-02-01',
			to: '2026-02-28',
		})
	})

	it('month view: a 31-day month', () => {
		expect(computeWindow('month', '2026-01-15')).toEqual({
			from: '2026-01-01',
			to: '2026-01-31',
		})
	})
})

describe('shiftDate', () => {
	it('moves forward and back one day in day view', () => {
		expect(shiftDate('day', '2026-08-07', 1)).toBe('2026-08-08')
		expect(shiftDate('day', '2026-08-07', -1)).toBe('2026-08-06')
	})

	it('crosses a month in day view', () => {
		expect(shiftDate('day', '2026-08-31', 1)).toBe('2026-09-01')
	})

	it('moves forward and back one week in week view', () => {
		expect(shiftDate('week', '2026-08-31', 1)).toBe('2026-09-07')
		expect(shiftDate('week', '2026-08-31', -1)).toBe('2026-08-24')
	})

	it('moves forward and back one month in month view', () => {
		expect(shiftDate('month', '2026-08-15', 1)).toBe('2026-09-15')
		expect(shiftDate('month', '2026-08-15', -1)).toBe('2026-07-15')
	})

	it('crosses a year in month view', () => {
		expect(shiftDate('month', '2025-12-15', 1)).toBe('2026-01-15')
		expect(shiftDate('month', '2026-01-15', -1)).toBe('2025-12-15')
	})
})

describe('enumerateDays', () => {
	it('enumerates every inclusive day of the window', () => {
		expect(enumerateDays('2026-08-03', '2026-08-05')).toEqual([
			'2026-08-03',
			'2026-08-04',
			'2026-08-05',
		])
	})

	it('returns a single day when from == to', () => {
		expect(enumerateDays('2026-08-07', '2026-08-07')).toEqual(['2026-08-07'])
	})

	it('covers a 31-day month', () => {
		expect(enumerateDays('2026-01-01', '2026-01-31')).toHaveLength(31)
	})
})
