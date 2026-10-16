import { describe, expect, it } from 'vitest'
import {
	lineSummaries,
	placeLabel,
	splitDrafts,
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

describe('splitDrafts', () => {
	function row(id: string, contract: Partial<MaintenanceContract>) {
		return {
			id,
			contract: { status: 'ACTIVE', ...contract } as MaintenanceContract,
		}
	}

	it('puts the draft contracts apart and keeps the others, in order', () => {
		const rows = [
			row('a', { status: 'ACTIVE' }),
			row('b', { status: 'DRAFT' }),
			row('c', { status: 'ACTIVE' }),
			row('d', { status: 'DRAFT' }),
		]

		const { drafts, active } = splitDrafts(rows)

		expect(drafts.map((entry) => entry.id)).toEqual(['b', 'd'])
		expect(active.map((entry) => entry.id)).toEqual(['a', 'c'])
	})

	it('leaves an archived draft in the main list: there is nothing left to review', () => {
		const rows = [
			row('a', { status: 'DRAFT', archived_at: '2026-10-01T00:00:00Z' }),
			row('b', { status: 'DRAFT', archived_at: null }),
		]

		const { drafts, active } = splitDrafts(rows)

		expect(drafts.map((entry) => entry.id)).toEqual(['b'])
		expect(active.map((entry) => entry.id)).toEqual(['a'])
	})

	it('has no draft when there is none', () => {
		expect(splitDrafts([row('a', {})]).drafts).toEqual([])
		expect(splitDrafts([])).toEqual({ drafts: [], active: [] })
	})
})
