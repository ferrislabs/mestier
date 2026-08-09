import { describe, expect, it } from 'vitest'
import { formatWindowLabel } from '#/pages/planning/lib/format'

describe('formatWindowLabel', () => {
	it('day view: long date, capitalised', () => {
		expect(formatWindowLabel('day', '2026-08-07', '2026-08-07')).toBe(
			'Vendredi 7 août 2026',
		)
	})

	it('week view: short interval of both bounds', () => {
		expect(formatWindowLabel('week', '2026-08-03', '2026-08-09')).toBe(
			'3 août – 9 août',
		)
	})

	it('month view: month and year, capitalised', () => {
		expect(formatWindowLabel('month', '2026-08-01', '2026-08-31')).toBe(
			'Août 2026',
		)
	})
})
