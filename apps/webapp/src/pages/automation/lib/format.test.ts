import { describe, expect, it } from 'vitest'
import {
	formatAgo,
	formatDuration,
	formatItemCount,
	formatMilliseconds,
	formatStartedAt,
	formatUsage,
	minutesSince,
} from '#/pages/automation/lib/format'

describe('format', () => {
	it('formatAgo renders minutes, hours and days in French', () => {
		expect(formatAgo(0)).toBe('à l’instant')
		expect(formatAgo(12)).toBe('il y a 12 min')
		expect(formatAgo(120)).toBe('il y a 2 h')
		expect(formatAgo(1440)).toBe('il y a 1 jour')
		expect(formatAgo(4320)).toBe('il y a 3 jours')
	})

	it('formatDuration uses a decimal comma', () => {
		expect(formatDuration(4.8)).toBe('4,8 s')
		expect(formatDuration(3)).toBe('3 s')
	})

	it('formatItemCount pluralises', () => {
		expect(formatItemCount(0)).toBe('0 élément')
		expect(formatItemCount(1)).toBe('1 élément')
		expect(formatItemCount(3)).toBe('3 éléments')
	})

	it('formatUsage describes workflow usage', () => {
		expect(formatUsage(0)).toBe('Non utilisée')
		expect(formatUsage(1)).toBe('Utilisée par 1 workflow')
		expect(formatUsage(3)).toBe('Utilisée par 3 workflows')
	})

	it('formatMilliseconds picks a readable unit', () => {
		expect(formatMilliseconds(180)).toBe('180 ms')
		expect(formatMilliseconds(4800)).toBe('4,8 s')
		expect(formatMilliseconds(125_000)).toBe('2 min 5 s')
		expect(formatMilliseconds(3 * 3_600_000 + 12 * 60_000)).toBe('3 h 12 min')
		expect(formatMilliseconds(30 * 86_400_000)).toBe('30 j')
	})

	it('minutesSince measures elapsed minutes and never goes negative', () => {
		const now = Date.parse('2026-08-01T01:00:00Z')
		expect(minutesSince('2026-08-01T00:00:00Z', now)).toBe(60)
		expect(minutesSince('2026-08-01T02:00:00Z', now)).toBe(0)
		expect(minutesSince('not a date', now)).toBe(0)
	})

	it('formatStartedAt says today and yesterday, otherwise gives the date', () => {
		const now = new Date(2026, 10, 5, 12, 0, 0).getTime()
		expect(
			formatStartedAt(new Date(2026, 10, 5, 9, 4, 3).toISOString(), now),
		).toBe('Aujourd’hui à 09:04:03')
		expect(
			formatStartedAt(new Date(2026, 10, 4, 23, 0, 0).toISOString(), now),
		).toBe('Hier à 23:00:00')
		expect(
			formatStartedAt(new Date(2026, 9, 1, 8, 0, 0).toISOString(), now),
		).toBe('1 oct. 2026 à 08:00:00')
		expect(formatStartedAt('nope', now)).toBe('—')
	})
})
