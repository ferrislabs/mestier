import { describe, expect, it } from 'vitest'
import {
	formatWeekStart,
	sortedAdoption,
	weeklyBars,
} from '#/pages/admin-stats/lib/stats'

describe('weeklyBars', () => {
	it('scales each week against the busiest one', () => {
		const bars = weeklyBars([
			{ week_start: '2026-11-02', count: 2 },
			{ week_start: '2026-11-09', count: 4 },
		])
		expect(bars.map((bar) => bar.percent)).toEqual([50, 100])
		expect(bars.map((bar) => bar.count)).toEqual([2, 4])
	})

	it('draws flat bars when nothing happened', () => {
		const bars = weeklyBars([{ week_start: '2026-11-02', count: 0 }])
		expect(bars[0]?.percent).toBe(0)
	})

	it('keeps the order of the weeks, oldest first', () => {
		const bars = weeklyBars([
			{ week_start: '2026-11-02', count: 1 },
			{ week_start: '2026-11-09', count: 1 },
		])
		expect(bars.map((bar) => bar.label)).toEqual(
			['2026-11-02', '2026-11-09'].map(formatWeekStart),
		)
	})
})

describe('formatWeekStart', () => {
	it('prints the day and the short month in French', () => {
		expect(formatWeekStart('2026-11-09')).toBe('9 nov.')
	})
})

describe('sortedAdoption', () => {
	it('puts the most adopted flag first', () => {
		const sorted = sortedAdoption([
			{ key: 'b', label: 'B', organizations: 1 },
			{ key: 'a', label: 'A', organizations: 3 },
		])
		expect(sorted.map((flag) => flag.key)).toEqual(['a', 'b'])
	})
})
