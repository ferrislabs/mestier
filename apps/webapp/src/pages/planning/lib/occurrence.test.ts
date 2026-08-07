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
	it("une entrée d'un seul jour n'occupe que ce jour-là", () => {
		const s = span('2026-08-10T08:00:00Z', '2026-08-10T10:00:00Z')
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-09', TZ)).toBe(false)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(false)
	})

	it('une absence de plusieurs jours occupe chacun de ses jours', () => {
		// Congé du 10 au 14 août (bornes locales Paris).
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

	it('une entrée journée entière ne déborde pas sur le lendemain (borne exclusive)', () => {
		const s = span(
			'2026-08-10T00:00:00+02:00',
			'2026-08-11T00:00:00+02:00',
			true,
		)
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(false)
	})

	it('une entrée à cheval sur minuit local occupe les deux jours', () => {
		// 22:00 Paris à 01:00 Paris le lendemain.
		const s = span('2026-08-10T20:00:00Z', '2026-08-10T23:00:00Z')
		expect(entryOccursOnDate(s, '2026-08-10', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-11', TZ)).toBe(true)
		expect(entryOccursOnDate(s, '2026-08-12', TZ)).toBe(false)
	})
})

describe('minuteSpanOnDate', () => {
	it('renvoie 0–1440 pour une entrée journée entière', () => {
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

	it('renvoie les minutes exactes pour une entrée entièrement dans le jour', () => {
		// 08:00–10:00 Paris.
		const s = span('2026-08-10T06:00:00Z', '2026-08-10T08:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 8 * 60,
			endMinute: 10 * 60,
		})
	})

	it('coupe à 00:00 une entrée qui a commencé la veille', () => {
		// 22:00 la veille à 01:00 le jour même (Paris) → sur le jour, 00:00–01:00.
		const s = span('2026-08-09T20:00:00Z', '2026-08-09T23:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 0,
			endMinute: 60,
		})
	})

	it('coupe à 24:00 une entrée qui déborde sur le lendemain', () => {
		// 22:00 Paris à 01:00 le lendemain → sur le jour, 22:00–24:00.
		const s = span('2026-08-10T20:00:00Z', '2026-08-10T23:00:00Z')
		expect(minuteSpanOnDate(s, '2026-08-10', TZ)).toEqual({
			startMinute: 22 * 60,
			endMinute: 24 * 60,
		})
	})
})
