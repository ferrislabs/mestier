import { describe, expect, it } from 'vitest'
import {
	type CadenceDraft,
	cadenceError,
	cadenceToDraft,
	describeCadence,
	emptyCadenceDraft,
	toCadenceRequest,
} from '#/pages/maintenance/lib/cadence-form'
import type { Cadence } from '#/pages/maintenance/types'

const SEASON = { firstWeek: 12, lastWeek: 48 }

function windows(...pairs: [string, string][]): CadenceDraft {
	return {
		...emptyCadenceDraft(),
		kind: 'WINDOWS',
		windows: pairs.map(([from, to]) => ({ from, to })),
	}
}

describe('cadenceError, every N days', () => {
	it('accepts 1 to 365', () => {
		for (const everyDays of ['1', '14', '365']) {
			expect(
				cadenceError({ ...emptyCadenceDraft(), everyDays }, SEASON),
			).toBeNull()
		}
	})

	it('refuses 0, 366, a decimal and an empty field', () => {
		for (const everyDays of ['0', '366', '2.5', '', 'deux']) {
			expect(cadenceError({ ...emptyCadenceDraft(), everyDays }, SEASON)).toBe(
				"L'intervalle va de 1 à 365 jours.",
			)
		}
	})
})

describe('cadenceError, windows', () => {
	it('accepts windows inside the season, apart from each other', () => {
		expect(
			cadenceError(windows(['14', '17'], ['27', '30'], ['44', '47']), SEASON),
		).toBeNull()
	})

	it('accepts a window of one week', () => {
		expect(cadenceError(windows(['20', '20']), SEASON)).toBeNull()
	})

	it('needs at least one window', () => {
		expect(cadenceError(windows(), SEASON)).toBe(
			'Ajoutez au moins une fenêtre.',
		)
	})

	it('refuses more than twelve windows', () => {
		const many = Array.from({ length: 13 }, (_, index): [string, string] => [
			String(13 + index * 2),
			String(13 + index * 2),
		])

		expect(cadenceError(windows(...many), SEASON)).toBe(
			'Une prestation a au plus 12 fenêtres.',
		)
	})

	it('refuses a window that ends before it starts', () => {
		expect(cadenceError(windows(['30', '27']), SEASON)).toBe(
			'La fenêtre S30 à S27 finit avant de commencer.',
		)
	})

	it('refuses a window outside the season', () => {
		expect(cadenceError(windows(['10', '14']), SEASON)).toBe(
			'La fenêtre S10 à S14 sort de la saison (S12 à S48).',
		)
		expect(cadenceError(windows(['45', '50']), SEASON)).toBe(
			'La fenêtre S45 à S50 sort de la saison (S12 à S48).',
		)
	})

	it('refuses windows that overlap, whatever the order they were typed in', () => {
		expect(cadenceError(windows(['20', '25'], ['25', '30']), SEASON)).toBe(
			'Deux fenêtres se chevauchent.',
		)
		expect(cadenceError(windows(['30', '35'], ['20', '31']), SEASON)).toBe(
			'Deux fenêtres se chevauchent.',
		)
	})

	it('refuses a blank bound', () => {
		expect(cadenceError(windows(['', '20']), SEASON)).toBe(
			'Chaque fenêtre a une semaine de début et une de fin.',
		)
	})
})

describe('cadenceError, on demand', () => {
	it('has nothing to check', () => {
		expect(
			cadenceError({ ...emptyCadenceDraft(), kind: 'ON_DEMAND' }, SEASON),
		).toBeNull()
	})
})

describe('toCadenceRequest', () => {
	it('sends each kind in the shape of the API', () => {
		expect(
			toCadenceRequest({ ...emptyCadenceDraft(), everyDays: '21' }),
		).toEqual({ kind: 'ROLLING', every_days: 21 })
		expect(toCadenceRequest(windows(['14', '17'], ['44', '47']))).toEqual({
			kind: 'WINDOWS',
			windows: [
				{ from_week: 14, to_week: 17 },
				{ from_week: 44, to_week: 47 },
			],
		})
		expect(
			toCadenceRequest({ ...emptyCadenceDraft(), kind: 'ON_DEMAND' }),
		).toEqual({ kind: 'ON_DEMAND' })
	})
})

describe('cadenceToDraft', () => {
	it('round-trips what the API returned', () => {
		const rolling: Cadence = { kind: 'ROLLING', every_days: 7 }
		const byWindows: Cadence = {
			kind: 'WINDOWS',
			windows: [{ from_week: 14, to_week: 17 }],
		}

		expect(toCadenceRequest(cadenceToDraft(rolling))).toEqual(rolling)
		expect(toCadenceRequest(cadenceToDraft(byWindows))).toEqual(byWindows)
		expect(toCadenceRequest(cadenceToDraft({ kind: 'ON_DEMAND' }))).toEqual({
			kind: 'ON_DEMAND',
		})
	})
})

describe('describeCadence', () => {
	it('reads like the sheet', () => {
		expect(describeCadence({ kind: 'ROLLING', every_days: 14 })).toBe(
			'Tous les 14 jours',
		)
		expect(describeCadence({ kind: 'ROLLING', every_days: 1 })).toBe(
			'Tous les jours',
		)
		expect(
			describeCadence({
				kind: 'WINDOWS',
				windows: [
					{ from_week: 14, to_week: 17 },
					{ from_week: 44, to_week: 44 },
				],
			}),
		).toBe('S14-S17, S44')
		expect(describeCadence({ kind: 'ON_DEMAND' })).toBe('À la demande')
	})
})
