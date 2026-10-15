import { describe, expect, it } from 'vitest'
import type { MaintenanceGrid } from '#/hooks/use-maintenance-validation'
import {
	describeVisit,
	interventionActionError,
	seasonsOf,
	visitLineIndex,
} from '#/pages/maintenance/lib/validation-lines'

const GRID: MaintenanceGrid = {
	year: 2026,
	contracts: [
		{
			id: 'contract-1',
			customer_id: 'customer-1',
			customer_context_id: 'context-1',
			reference: 'C25029',
			season: { year: 2026, first_week: 12, last_week: 48 },
			status: 'ACTIVE',
			site: { customer_name: 'Gaume', site_label: 'Siège', city: 'Vichy' },
			lines: [
				{
					id: 'line-1',
					label: 'Tonte',
					cadence: { kind: 'ON_DEMAND' },
					due: { status: 'DONE' },
					visits: [],
				},
			],
		},
	] as MaintenanceGrid['contracts'],
}

describe('seasonsOf', () => {
	it('lists each year once, in order', () => {
		expect(
			seasonsOf([
				{ done_on: '2026-10-14' },
				{ done_on: '2025-12-30' },
				{ done_on: '2026-03-02' },
			]),
		).toEqual([2025, 2026])
	})

	it('is empty for an empty queue', () => {
		expect(seasonsOf([])).toEqual([])
	})
})

describe('visitLineIndex', () => {
	it('finds a line by id, with its site', () => {
		const index = visitLineIndex([GRID])

		expect(index.get('line-1')).toEqual({
			label: 'Tonte',
			site: 'Gaume · Siège (Vichy)',
			reference: 'C25029',
		})
		expect(index.get('line-9')).toBeUndefined()
	})

	it('ignores a grid that has not loaded', () => {
		expect(visitLineIndex([undefined]).size).toBe(0)
	})
})

describe('describeVisit', () => {
	it('names the line and the site', () => {
		const index = visitLineIndex([GRID])

		expect(describeVisit('line-1', index)).toEqual({
			title: 'Tonte',
			subtitle: 'Gaume · Siège (Vichy) · C25029',
		})
	})

	it('says plainly when the line is not in the grid', () => {
		expect(describeVisit('line-9', new Map())).toEqual({
			title: 'Prestation introuvable',
			subtitle: 'Son contrat est peut-être archivé.',
		})
	})
})

describe('interventionActionError', () => {
	it('explains a visit someone else already resolved', () => {
		expect(interventionActionError({ status: 409 })).toMatch(/déjà été traitée/)
	})

	it('explains a visit that is gone', () => {
		expect(interventionActionError({ status: 404 })).toMatch(/n’existe plus/)
	})

	it('has nothing to say when nothing failed', () => {
		expect(interventionActionError(null)).toBeNull()
	})
})
