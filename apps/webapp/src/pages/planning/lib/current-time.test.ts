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
	it('calcule le délai restant jusqu’à la prochaine frontière de minute', () => {
		const now = new Date('2026-08-10T12:30:15.250Z')
		expect(millisecondsUntilNextMinute(now)).toBe(60_000 - 15_250)
	})

	it('renvoie une minute pleine pile sur une frontière de minute', () => {
		const now = new Date('2026-08-10T12:31:00.000Z')
		expect(millisecondsUntilNextMinute(now)).toBe(60_000)
	})

	it('renvoie une valeur toujours strictement positive et au plus 60000', () => {
		for (const seconds of [0, 1, 29, 59, 59.9]) {
			const now = new Date(2026, 7, 10, 12, 30, Math.floor(seconds), 0)
			const delay = millisecondsUntilNextMinute(now)
			expect(delay).toBeGreaterThan(0)
			expect(delay).toBeLessThanOrEqual(60_000)
		}
	})
})

describe('isCurrentTimeVisible', () => {
	it('absent quand le jour courant est hors de la période visible', () => {
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

	it("absent quand l'heure courante est avant l'amplitude", () => {
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

	it("absent quand l'heure courante est après l'amplitude", () => {
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

	it("présent quand le jour est dans la période et l'heure dans l'amplitude", () => {
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

	it('présent aux bornes inclusives de l’amplitude', () => {
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
	it('place l’heure courante proportionnellement le long de l’amplitude', () => {
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

	it('trouve l’index de la colonne du jour courant', () => {
		expect(
			findTodayColumnIndex(columns, new Date('2026-08-05T11:00:00Z'), TZ),
		).toBe(2)
	})

	it('renvoie null quand le jour courant n’est pas dans les colonnes', () => {
		expect(
			findTodayColumnIndex(columns, new Date('2026-08-20T11:00:00Z'), TZ),
		).toBeNull()
	})
})
