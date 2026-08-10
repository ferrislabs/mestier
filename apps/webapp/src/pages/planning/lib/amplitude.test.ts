import { describe, expect, it } from 'vitest'
import {
	computeAmplitude,
	FALLBACK_AMPLITUDE,
	isoDateInTimeZone,
	minuteOfDayInTimeZone,
} from '#/pages/planning/lib/amplitude'

const TZ = 'Europe/Paris'

describe('minuteOfDayInTimeZone', () => {
	it("converts a UTC instant into minutes since midnight in the organization's zone", () => {
		// 06:30 UTC is 08:30 in Paris in summer (UTC+2).
		expect(minuteOfDayInTimeZone(new Date('2026-08-10T06:30:00Z'), TZ)).toBe(
			8 * 60 + 30,
		)
	})

	it("handles a zone different from the browser's without depending on it", () => {
		expect(
			minuteOfDayInTimeZone(
				new Date('2026-08-10T06:30:00Z'),
				'America/New_York',
			),
		).toBe(2 * 60 + 30)
	})
})

describe('isoDateInTimeZone', () => {
	it('returns the calendar date local to the organization, not the UTC one', () => {
		// 23:15 UTC is already the next day in Paris (01:15, summer).
		expect(isoDateInTimeZone(new Date('2026-08-10T23:15:00Z'), TZ)).toBe(
			'2026-08-11',
		)
	})
})

describe('computeAmplitude', () => {
	it('nominal case: bounds rounded to the hour over the work ranges', () => {
		const amplitude = computeAmplitude(
			[],
			[
				{ startsMinute: 9 * 60, endsMinute: 12 * 60 + 30 },
				{ startsMinute: 13 * 60 + 15, endsMinute: 17 * 60 },
			],
			TZ,
		)
		expect(amplitude).toEqual({ startMinute: 9 * 60, endMinute: 17 * 60 })
	})

	it('rounds the lower bound down and the upper bound up', () => {
		const amplitude = computeAmplitude(
			[],
			[{ startsMinute: 8 * 60 + 45, endsMinute: 17 * 60 + 10 }],
			TZ,
		)
		expect(amplitude).toEqual({ startMinute: 8 * 60, endMinute: 18 * 60 })
	})

	it('falls back to 08:00–18:00 when the period is empty', () => {
		expect(computeAmplitude([], [], TZ)).toEqual(FALLBACK_AMPLITUDE)
		expect(FALLBACK_AMPLITUDE).toEqual({
			startMinute: 8 * 60,
			endMinute: 18 * 60,
		})
	})

	it('an entry spilling past the work ranges widens the amplitude', () => {
		const amplitude = computeAmplitude(
			[
				{
					startsAt: '2026-08-10T05:15:00Z', // 07:15 Paris
					endsAt: '2026-08-10T18:40:00Z', // 20:40 Paris
					allDay: false,
				},
			],
			[{ startsMinute: 9 * 60, endsMinute: 17 * 60 }],
			TZ,
		)
		expect(amplitude).toEqual({ startMinute: 7 * 60, endMinute: 21 * 60 })
	})

	it('excludes full-day entries from the computation — otherwise a single absence would open the amplitude to 24h', () => {
		const amplitude = computeAmplitude(
			[
				{
					startsAt: '2026-08-10T00:00:00Z',
					endsAt: '2026-08-11T00:00:00Z',
					allDay: true,
				},
			],
			[],
			TZ,
		)
		expect(amplitude).toEqual(FALLBACK_AMPLITUDE)
	})

	it('an entry straddling local midnight extends the upper bound to 24:00', () => {
		const amplitude = computeAmplitude(
			[
				{
					// 22:00 Paris to 01:00 Paris the next day.
					startsAt: '2026-08-10T20:00:00Z',
					endsAt: '2026-08-10T23:00:00Z',
					allDay: false,
				},
			],
			[],
			TZ,
		)
		expect(amplitude).toEqual({ startMinute: 22 * 60, endMinute: 24 * 60 })
	})
})
