import { describe, expect, it } from 'vitest'
import {
	computeWindow,
	enumerateDays,
	shiftDate,
} from '#/pages/planning/lib/window'

describe('computeWindow', () => {
	it('vue jour : la fenêtre est le jour lui-même', () => {
		expect(computeWindow('day', '2026-08-07')).toEqual({
			from: '2026-08-07',
			to: '2026-08-07',
		})
	})

	it('vue semaine : du lundi au dimanche contenant la date', () => {
		// 2026-08-07 est un vendredi.
		expect(computeWindow('week', '2026-08-07')).toEqual({
			from: '2026-08-03',
			to: '2026-08-09',
		})
	})

	it('vue semaine : bascule de mois quand la semaine chevauche deux mois', () => {
		// 2026-08-31 est un lundi ; la semaine se termine en septembre.
		expect(computeWindow('week', '2026-08-31')).toEqual({
			from: '2026-08-31',
			to: '2026-09-06',
		})
	})

	it("vue semaine : bascule d'année quand la semaine chevauche deux années", () => {
		// 2025-12-29 est un lundi ; la semaine se termine en janvier 2026.
		expect(computeWindow('week', '2025-12-29')).toEqual({
			from: '2025-12-29',
			to: '2026-01-04',
		})
		expect(computeWindow('week', '2026-01-01')).toEqual({
			from: '2025-12-29',
			to: '2026-01-04',
		})
	})

	it('vue mois : du premier au dernier jour du mois contenant la date', () => {
		expect(computeWindow('month', '2026-02-15')).toEqual({
			from: '2026-02-01',
			to: '2026-02-28',
		})
	})

	it('vue mois : un mois de 31 jours', () => {
		expect(computeWindow('month', '2026-01-15')).toEqual({
			from: '2026-01-01',
			to: '2026-01-31',
		})
	})
})

describe('shiftDate', () => {
	it('avance et recule d’un jour en vue jour', () => {
		expect(shiftDate('day', '2026-08-07', 1)).toBe('2026-08-08')
		expect(shiftDate('day', '2026-08-07', -1)).toBe('2026-08-06')
	})

	it('bascule de mois en vue jour', () => {
		expect(shiftDate('day', '2026-08-31', 1)).toBe('2026-09-01')
	})

	it('avance et recule d’une semaine en vue semaine', () => {
		expect(shiftDate('week', '2026-08-31', 1)).toBe('2026-09-07')
		expect(shiftDate('week', '2026-08-31', -1)).toBe('2026-08-24')
	})

	it('avance et recule d’un mois en vue mois', () => {
		expect(shiftDate('month', '2026-08-15', 1)).toBe('2026-09-15')
		expect(shiftDate('month', '2026-08-15', -1)).toBe('2026-07-15')
	})

	it("bascule d'année en vue mois", () => {
		expect(shiftDate('month', '2025-12-15', 1)).toBe('2026-01-15')
		expect(shiftDate('month', '2026-01-15', -1)).toBe('2025-12-15')
	})
})

describe('enumerateDays', () => {
	it('énumère tous les jours inclusifs de la fenêtre', () => {
		expect(enumerateDays('2026-08-03', '2026-08-05')).toEqual([
			'2026-08-03',
			'2026-08-04',
			'2026-08-05',
		])
	})

	it('renvoie un seul jour quand from == to', () => {
		expect(enumerateDays('2026-08-07', '2026-08-07')).toEqual(['2026-08-07'])
	})

	it('couvre un mois de 31 jours', () => {
		expect(enumerateDays('2026-01-01', '2026-01-31')).toHaveLength(31)
	})
})
