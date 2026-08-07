import { describe, expect, it } from 'vitest'
import {
	computeSegmentPosition,
	stackOverlapping,
} from '#/pages/planning/lib/layout'

const AXIS = { startMinute: 8 * 60, endMinute: 18 * 60 } // 08:00–18:00

describe('computeSegmentPosition', () => {
	it('positionne un segment entièrement dans l’axe', () => {
		expect(
			computeSegmentPosition({ startMinute: 9 * 60, endMinute: 11 * 60 }, AXIS),
		).toEqual({ left: 10, width: 20 })
	})

	it('couvre 100% de la largeur pour une entrée journée entière (00:00–24:00)', () => {
		expect(
			computeSegmentPosition({ startMinute: 0, endMinute: 24 * 60 }, AXIS),
		).toEqual({ left: 0, width: 100 })
	})

	it('coupe à gauche une entrée qui commence avant l’axe', () => {
		// 07:00–11:00, axe 08:00–18:00 → visible seulement à partir de 08:00.
		expect(
			computeSegmentPosition({ startMinute: 7 * 60, endMinute: 11 * 60 }, AXIS),
		).toEqual({ left: 0, width: 30 })
	})

	it('coupe à droite une entrée qui dépasse la fin de l’axe', () => {
		// 15:00–19:00, axe 08:00–18:00 → visible jusqu'à 18:00 seulement.
		expect(
			computeSegmentPosition(
				{ startMinute: 15 * 60, endMinute: 19 * 60 },
				AXIS,
			),
		).toEqual({ left: 70, width: 30 })
	})

	it('renvoie une largeur nulle pour un segment entièrement hors axe', () => {
		expect(
			computeSegmentPosition({ startMinute: 0, endMinute: 60 }, AXIS),
		).toEqual({ left: 0, width: 0 })
	})

	it('renvoie une largeur nulle quand l’axe est dégénéré', () => {
		expect(
			computeSegmentPosition(
				{ startMinute: 9 * 60, endMinute: 10 * 60 },
				{ startMinute: 8 * 60, endMinute: 8 * 60 },
			),
		).toEqual({ left: 0, width: 0 })
	})
})

describe('stackOverlapping', () => {
	it('empile deux entrées qui se chevauchent sur deux lignes', () => {
		const items = [
			{ id: 'a', startMinute: 9 * 60, endMinute: 11 * 60 },
			{ id: 'b', startMinute: 10 * 60, endMinute: 12 * 60 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))
		const rowOf = (id: string) => result.find((r) => r.item.id === id)?.row

		expect(rowOf('a')).toBe(0)
		expect(rowOf('b')).toBe(1)
	})

	it('partage la même ligne pour deux entrées consécutives non chevauchantes', () => {
		const items = [
			{ id: 'a', startMinute: 9 * 60, endMinute: 10 * 60 },
			{ id: 'b', startMinute: 10 * 60, endMinute: 11 * 60 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))

		expect(result.every((r) => r.row === 0)).toBe(true)
	})

	it('réutilise une ligne libérée plutôt que d’en ouvrir une nouvelle', () => {
		// A(0-60) et B(30-90) se chevauchent ; C(60-120) chevauche B mais pas A,
		// donc C peut reprendre la ligne 0 libérée par la fin de A.
		const items = [
			{ id: 'a', startMinute: 0, endMinute: 60 },
			{ id: 'b', startMinute: 30, endMinute: 90 },
			{ id: 'c', startMinute: 60, endMinute: 120 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))
		const rowOf = (id: string) => result.find((r) => r.item.id === id)?.row

		expect(rowOf('a')).toBe(0)
		expect(rowOf('b')).toBe(1)
		expect(rowOf('c')).toBe(0)
	})

	it('ne modifie pas le tableau reçu en entrée', () => {
		const items = [
			{ id: 'b', startMinute: 10 * 60, endMinute: 11 * 60 },
			{ id: 'a', startMinute: 9 * 60, endMinute: 10 * 60 },
		]
		stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))

		expect(items[0]?.id).toBe('b')
		expect(items[1]?.id).toBe('a')
	})
})
