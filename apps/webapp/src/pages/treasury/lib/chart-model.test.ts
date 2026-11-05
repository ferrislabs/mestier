import { describe, expect, it } from 'vitest'
import {
	niceCeiling,
	trendDescription,
	trendModel,
} from '#/pages/treasury/lib/chart-model'

describe('niceCeiling', () => {
	it('rounds up to a readable axis top', () => {
		expect(niceCeiling(0)).toBe(1)
		expect(niceCeiling(480)).toBe(500)
		expect(niceCeiling(1200)).toBe(2000)
		expect(niceCeiling(100)).toBe(100)
	})
})

describe('trendModel', () => {
	const trend = [
		{ month: '2026-10', receipts: 100000, expenses: 40000 },
		{ month: '2026-11', receipts: 0, expenses: 20000 },
	]

	it('scales bars against the axis top', () => {
		const model = trendModel(trend)
		expect(model.top).toBe(100000)
		expect(model.ticks.map((tick) => tick.ratio)).toEqual([0, 0.5, 1])
		expect(model.bars[0]?.receiptsRatio).toBe(1)
		expect(model.bars[1]?.expensesRatio).toBe(0.2)
	})

	it('describes every month for assistive technology', () => {
		expect(trendDescription(trend)).toContain('octobre 2026')
	})
})
