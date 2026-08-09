import { describe, expect, it } from 'vitest'
import type { TimeSpan } from '#/pages/planning/lib/amplitude'
import {
	entryOccursOnDate,
	minuteSpanOnDate,
} from '#/pages/planning/lib/occurrence'

const TZ = 'Europe/Paris'

function span(startsAt: string, endsAt: string, allDay = false): TimeSpan {
	return { startsAt, endsAt, allDay }
}

describe('entryOccursOnDate', () => {
	it('a single-day entry occupies that day only', () => {
		const s = span('2026-08-10T08:00:00Z', '2026-08-10T10:00:00Z')
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-09', TZ)).toBe(false)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(false)
	})

	it('a multi-day absence occupies each of its days', () => {
		// Leave from 10 to 14 August (local Paris bounds).
		const s = span(
			'2026-08-10T00:00:00+02:00',
			'2026-08-15T00:00:00+02:00',
			true,
		)
		expect(entryOccursOnDate(s, '2026-08-09', TZ)).toBe(false)
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-12', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-14', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-15', TZ)).toBe(false)
	})

	it('a full-day entry does not spill onto the next day (exclusive bound)', () => {
		const s = span(
			'2026-08-10T00:00:00+02:00',
			'2026-08-11T00:00:00+02:00',
			true,
		)
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(false)
	})

	it('an entry straddling local midnight occupies both days', () => {
		// 22:00 Paris to 01:00 Paris the next day.
		const s = span('2026-08-10T20:00:00Z', '2026-08-10T23:00:00Z')
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-12', TZ)).toBe(false)
	})
})

describe('minuteSpanOnDate', () => {
	it('returns 0–1440 for a full-day entry', () => {
		const s = span(
			'2026-08-10T00:00:00+02:00',
			'2026-08-11T00:00:00+02:00',
			true,
		)
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 0,
			endMinute: 1440,
		})
	})

	it('returns the exact minutes for an entry fully inside the day', () => {
		// 08:00–10:00 Paris.
		const s = span('2026-08-10T06:00:00Z', '2026-08-10T08:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 8 * 60,
			endMinute: 10 * 60,
		})
	})

	it('clips at 00:00 an entry that started the day before', () => {
		// 22:00 the day before to 01:00 the same day (Paris) → on that day, 00:00–01:00.
		const s = span('2026-08-09T20:00:00Z', '2026-08-09T23:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 0,
			endMinute: 60,
		})
	})

	it('clips at 24:00 an entry spilling onto the next day', () => {
		// 22:00 Paris to 01:00 the next day → on that day, 22:00–24:00.
		const s = span('2026-08-10T20:00:00Z', '2026-08-10T23:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 22 * 60,
			endMinute: 24 * 60,
		})
	})
})
