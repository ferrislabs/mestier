import { describe, expect, it } from 'vitest'
import {
	currentTimePercent,
	findTodayColumnIndex,
	isCurrentTimeVisible,
	millisecondsUntilNextMinute,
} from '#/pages/planning/lib/current-time'

const TZ = 'Europe/Paris'
const AMPLITUDE = { startMinute: 8 * 60, endMinute: 18 * 60 }

describe('millisecondsUntilNextMinute', () => {
	it('computes the delay left until the next minute boundary', () => {
		const now = new Date('2026-08-10T12:30:15.250Z')
		expect(millisecondsUntilNextMinute(now)).toBe(60_000 - 15_250)
	})

	it('returns a whole minute exactly on a minute boundary', () => {
		const now = new Date('2026-08-10T12:31:00.000Z')
		expect(millisecondsUntilNextMinute(now)).toBe(60_000)
	})

	it('returns a value always strictly positive and at most 60000', () => {
		for (const seconds of [0, 1, 29, 59, 59.9]) {
			const now = new Date(2026, 7, 10, 12, 30, Math.floor(seconds), 0)
			const delay = millisecondsUntilNextMinute(now)
			expect(delay).toBeGreaterThan(0)
			expect(delay).toBeLessThanOrEqual(60_000)
		}
	})
})

describe('isCurrentTimeVisible', () => {
	it('absent when the current day is outside the visible period', () => {
		expect(
			isCurrentTimeVisible({
				now: new Date('2026-08-20T10:00:00Z'),
				timeZone: TZ,
				windowFrom: '2026-08-03',
				windowTo: '2026-08-09',
				amplitude: AMPLITUDE,
			}),
		).toBe(false)
	})

	it('absent when the current time is before the amplitude', () => {
		expect(
			isCurrentTimeVisible({
				now: new Date('2026-08-05T05:00:00Z'), // 07:00 Paris
				timeZone: TZ,
				windowFrom: '2026-08-03',
				windowTo: '2026-08-09',
				amplitude: AMPLITUDE,
			}),
		).toBe(false)
	})

	it('absent when the current time is after the amplitude', () => {
		expect(
			isCurrentTimeVisible({
				now: new Date('2026-08-05T17:30:00Z'), // 19:30 Paris
				timeZone: TZ,
				windowFrom: '2026-08-03',
				windowTo: '2026-08-09',
				amplitude: AMPLITUDE,
			}),
		).toBe(false)
	})

	it('present when the day is in the period and the time in the amplitude', () => {
		expect(
			isCurrentTimeVisible({
				now: new Date('2026-08-05T11:00:00Z'), // 13:00 Paris
				timeZone: TZ,
				windowFrom: '2026-08-03',
				windowTo: '2026-08-09',
				amplitude: AMPLITUDE,
			}),
		).toBe(true)
	})

	it("present at the amplitude's inclusive bounds", () => {
		const base = {
			timeZone: TZ,
			windowFrom: '2026-08-03',
			windowTo: '2026-08-09',
			amplitude: AMPLITUDE,
		}
		expect(
			isCurrentTimeVisible({ ...base, now: new Date('2026-08-05T06:00:00Z') }),
		).toBe(true) // 08:00 Paris pile
		expect(
			isCurrentTimeVisible({ ...base, now: new Date('2026-08-05T16:00:00Z') }),
		).toBe(true) // 18:00 Paris pile
	})
})

describe('currentTimePercent', () => {
	it('places the current time proportionally along the amplitude', () => {
		const now = new Date('2026-08-05T11:00:00Z') // 13:00 Paris
		expect(currentTimePercent(now, TZ, AMPLITUDE)).toBe(50)
	})
})

describe('findTodayColumnIndex', () => {
	const columns = [
		'2026-08-03',
		'2026-08-04',
		'2026-08-05',
		'2026-08-06',
		'2026-08-07',
	]

	it("finds the index of the current day's column", () => {
		expect(
			findTodayColumnIndex(columns, new Date('2026-08-05T11:00:00Z'), TZ),
		).toBe(2)
	})

	it('returns null when the current day is not among the columns', () => {
		expect(
			findTodayColumnIndex(columns, new Date('2026-08-20T11:00:00Z'), TZ),
		).toBeNull()
	})
})
