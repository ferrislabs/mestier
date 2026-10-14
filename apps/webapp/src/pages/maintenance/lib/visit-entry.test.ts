import { describe, expect, it } from 'vitest'
import { parseVisitDay } from '#/pages/maintenance/lib/visit-entry'

describe('parseVisitDay', () => {
	it('turns the day typed in a week into a date', () => {
		expect(parseVisitDay('21', 2026, 43)).toEqual({
			ok: true,
			date: '2026-10-21',
		})
	})

	it('takes the month from the week when it straddles two', () => {
		expect(parseVisitDay('1', 2026, 44)).toEqual({
			ok: true,
			date: '2026-11-01',
		})
	})

	it('refuses a day the week does not contain', () => {
		expect(parseVisitDay('5', 2026, 43)).toEqual({
			ok: false,
			error: "Le 5 n'est pas dans la semaine S43.",
		})
	})

	it('refuses anything that is not a day number', () => {
		for (const input of ['', ' ', 'vingt', '2026-10-21', '-3', '1.5']) {
			expect(parseVisitDay(input, 2026, 43).ok).toBe(false)
		}
	})
})
