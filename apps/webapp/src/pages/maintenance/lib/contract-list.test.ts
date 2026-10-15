import { describe, expect, it } from 'vitest'
import {
	lineSummaries,
	placeLabel,
} from '#/pages/maintenance/lib/contract-list'
import type { MaintenanceContract } from '#/pages/maintenance/types'

describe('placeLabel', () => {
	it('joins the customer and the site', () => {
		expect(placeLabel('GAUME', 'Jardin')).toBe('GAUME - Jardin')
	})

	it('does not say the same thing twice', () => {
		expect(placeLabel('GAUME', 'GAUME')).toBe('GAUME')
	})

	it('falls back on what is known while the names load', () => {
		expect(placeLabel('GAUME', null)).toBe('GAUME')
		expect(placeLabel(null, 'Jardin')).toBe('Jardin')
		expect(placeLabel(null, null)).toBe('Client inconnu')
	})
})

describe('lineSummaries', () => {
	it('puts the cadence next to each line', () => {
		const contract = {
			lines: [
				{
					id: 'l1',
					label: 'Tonte',
					cadence: { kind: 'ROLLING', every_days: 14 },
				},
				{ id: 'l2', label: 'ND', cadence: { kind: 'ON_DEMAND' } },
			],
		} as MaintenanceContract

		expect(lineSummaries(contract)).toEqual([
			'Tonte (Tous les 14 jours)',
			'ND (À la demande)',
		])
	})
})
