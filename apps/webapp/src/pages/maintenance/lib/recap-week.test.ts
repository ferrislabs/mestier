import { describe, expect, it } from 'vitest'
import {
	defaultRecapWeek,
	formatIsoWeek,
	isoWeekMonday,
	parseIsoWeek,
	recapSearchSchema,
	shiftIsoWeek,
	weekLabel,
} from '#/pages/maintenance/lib/recap-week'

describe('formatIsoWeek', () => {
	it('names a date by its ISO week, zero padded', () => {
		expect(formatIsoWeek(new Date(2026, 9, 19))).toBe('2026-W43')
		expect(formatIsoWeek(new Date(2026, 0, 7))).toBe('2026-W02')
	})

	it('uses the ISO year, not the calendar year, around New Year', () => {
		expect(formatIsoWeek(new Date(2025, 11, 29))).toBe('2026-W01')
		expect(formatIsoWeek(new Date(2027, 0, 1))).toBe('2026-W53')
	})
})

describe('defaultRecapWeek', () => {
	it('is next week on a Thursday, the day the recap is read', () => {
		expect(defaultRecapWeek(new Date(2026, 9, 15))).toBe('2026-W43')
	})

	it('is next week on any other day, Monday and Sunday included', () => {
		expect(defaultRecapWeek(new Date(2026, 9, 12))).toBe('2026-W43')
		expect(defaultRecapWeek(new Date(2026, 9, 18))).toBe('2026-W43')
		expect(defaultRecapWeek(new Date(2026, 9, 19))).toBe('2026-W44')
	})

	it('rolls over to week 1 of the next ISO year', () => {
		expect(defaultRecapWeek(new Date(2026, 11, 31))).toBe('2027-W01')
	})
})

describe('parseIsoWeek', () => {
	it('reads the year and the week', () => {
		expect(parseIsoWeek('2026-W43')).toEqual({ year: 2026, week: 43 })
		expect(parseIsoWeek('2026-W5')).toEqual({ year: 2026, week: 5 })
	})

	it('refuses a week the year does not have', () => {
		expect(parseIsoWeek('2026-W53')).toEqual({ year: 2026, week: 53 })
		expect(parseIsoWeek('2027-W53')).toBeNull()
		expect(parseIsoWeek('2026-W00')).toBeNull()
		expect(parseIsoWeek('2026-W54')).toBeNull()
	})

	it('refuses anything that is not an ISO week', () => {
		for (const raw of ['', '2026', '2026-43', '2026-w43', '26-W43', 'x']) {
			expect(parseIsoWeek(raw)).toBeNull()
		}
	})
})

describe('isoWeekMonday', () => {
	it('is the Monday of the week', () => {
		const monday = isoWeekMonday('2026-W43')
		expect(monday && formatDay(monday)).toBe('2026-10-19')
		expect(formatDay(isoWeekMonday('2026-W01') as Date)).toBe('2025-12-29')
	})

	it('is null for a bad week', () => {
		expect(isoWeekMonday('2027-W53')).toBeNull()
	})
})

describe('shiftIsoWeek', () => {
	it('moves by whole weeks', () => {
		expect(shiftIsoWeek('2026-W43', 1)).toBe('2026-W44')
		expect(shiftIsoWeek('2026-W43', -2)).toBe('2026-W41')
	})

	it('crosses the year boundary both ways', () => {
		expect(shiftIsoWeek('2026-W53', 1)).toBe('2027-W01')
		expect(shiftIsoWeek('2027-W01', -1)).toBe('2026-W53')
	})

	it('leaves an unreadable week as it was', () => {
		expect(shiftIsoWeek('nope', 1)).toBe('nope')
	})
})

describe('weekLabel', () => {
	it('gives the number and the days it covers', () => {
		expect(weekLabel('2026-W43')).toBe('S43 2026 · du 19 oct. au 25 oct.')
	})

	it('falls back to the raw value when it is not a week', () => {
		expect(weekLabel('nope')).toBe('nope')
	})
})

describe('recapSearchSchema', () => {
	it('keeps a week it can read', () => {
		expect(recapSearchSchema.parse({ week: '2026-W43' })).toEqual({
			week: '2026-W43',
		})
	})

	it('falls back to a readable week for a missing or broken one', () => {
		for (const search of [{}, { week: 'nope' }, { week: '2027-W53' }]) {
			const { week } = recapSearchSchema.parse(search)
			expect(parseIsoWeek(week)).not.toBeNull()
		}
	})
})

function formatDay(date: Date): string {
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${date.getFullYear()}-${month}-${day}`
}
