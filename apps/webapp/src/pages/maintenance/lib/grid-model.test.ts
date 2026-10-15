import { describe, expect, it } from 'vitest'
import {
	buildLineCells,
	contractTitle,
	sheetWeeks,
} from '#/pages/maintenance/lib/grid-model'
import type { GridContract, GridLine } from '#/pages/maintenance/types'

function line(overrides: Partial<GridLine> = {}): GridLine {
	return {
		id: 'line-1',
		label: 'Tonte',
		cadence: { kind: 'ROLLING', every_days: 14 },
		visits: [],
		due: { status: 'LATER' },
		...overrides,
	}
}

function contract(overrides: Partial<GridContract> = {}): GridContract {
	return {
		id: 'contract-1',
		reference: 'C25029',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		site: { customer_name: 'GAUME', site_label: 'Jardin' },
		season: { year: 2026, first_week: 40, last_week: 46 },
		status: 'ACTIVE',
		lines: [],
		...overrides,
	}
}

const WEEKS = [40, 41, 42, 43, 44, 45, 46]

function cell(cells: ReturnType<typeof buildLineCells>, week: number) {
	const found = cells.find((entry) => entry.week === week)
	if (!found) throw new Error(`no cell for week ${week}`)
	return found
}

describe('sheetWeeks', () => {
	it('falls back to the paper sheet when there is no contract', () => {
		const weeks = sheetWeeks([])

		expect(weeks[0]).toBe(12)
		expect(weeks.at(-1)).toBe(48)
		expect(weeks).toHaveLength(37)
	})

	it('spans the widest season among the contracts', () => {
		const weeks = sheetWeeks([
			contract({ season: { year: 2026, first_week: 14, last_week: 40 } }),
			contract({ season: { year: 2026, first_week: 12, last_week: 36 } }),
		])

		expect([weeks[0], weeks.at(-1)]).toEqual([12, 40])
	})
})

describe('buildLineCells', () => {
	it('places a visit in its week with the day of the month', () => {
		const cells = buildLineCells(
			contract(),
			line({
				visits: [
					{
						id: 'v1',
						done_on: '2026-10-21',
						week: 43,
						status: 'VALIDATED',
					},
				],
			}),
			WEEKS,
		)

		expect(cell(cells, 43)).toMatchObject({
			state: 'validated',
			clickable: false,
			visits: [{ day: 21 }],
		})
		expect(cell(cells, 42).state).toBe('empty')
	})

	it('distinguishes a proposed visit from a validated one', () => {
		const cells = buildLineCells(
			contract(),
			line({
				visits: [
					{ id: 'v1', done_on: '2026-10-07', week: 41, status: 'PROPOSED' },
				],
			}),
			WEEKS,
		)

		expect(cell(cells, 41).state).toBe('proposed')
	})

	it('leaves rejected visits off the sheet', () => {
		const cells = buildLineCells(
			contract(),
			line({
				visits: [
					{ id: 'v1', done_on: '2026-10-07', week: 41, status: 'REJECTED' },
				],
			}),
			WEEKS,
		)

		expect(cell(cells, 41)).toMatchObject({ state: 'empty', clickable: true })
	})

	it('places the due date given by the backend in its week', () => {
		const cells = buildLineCells(
			contract(),
			line({ due: { status: 'NEXT_WEEK', on: '2026-11-04' } }),
			WEEKS,
		)

		expect(cell(cells, 45)).toMatchObject({
			state: 'due',
			due: { date: '2026-11-04', day: 4 },
			clickable: true,
		})
	})

	it('maps the backend statuses onto planned, due and overdue', () => {
		const kindFor = (status: GridLine['due']['status']) =>
			cell(
				buildLineCells(
					contract(),
					line({ due: { status, on: '2026-10-07' } }),
					WEEKS,
				),
				41,
			).state

		expect(kindFor('PLANNED')).toBe('planned')
		expect(kindFor('OVERDUE')).toBe('overdue')
		expect(kindFor('THIS_WEEK')).toBe('due')
		expect(kindFor('LATER')).toBe('due')
	})

	it('shows no due cell once the line is done', () => {
		const cells = buildLineCells(
			contract(),
			line({ due: { status: 'DONE' } }),
			WEEKS,
		)

		expect(cells.every((entry) => entry.state === 'empty')).toBe(true)
	})

	it('keeps the due date visible under a proposed visit', () => {
		const cells = buildLineCells(
			contract(),
			line({
				visits: [
					{ id: 'v1', done_on: '2026-10-21', week: 43, status: 'PROPOSED' },
				],
				due: { status: 'THIS_WEEK', on: '2026-10-22' },
			}),
			WEEKS,
		)

		expect(cell(cells, 43)).toMatchObject({
			state: 'proposed',
			due: { day: 22 },
		})
	})

	it('draws the weeks outside the contract season as not clickable', () => {
		const cells = buildLineCells(
			contract({ season: { year: 2026, first_week: 41, last_week: 44 } }),
			line(),
			WEEKS,
		)

		expect(cell(cells, 40)).toMatchObject({
			state: 'out-of-season',
			clickable: false,
		})
		expect(cell(cells, 41).clickable).toBe(true)
		expect(cell(cells, 46).state).toBe('out-of-season')
	})

	it('ignores a due date that falls in another ISO year', () => {
		const cells = buildLineCells(
			contract(),
			line({ due: { status: 'LATER', on: '2027-10-20' } }),
			WEEKS,
		)

		expect(cells.every((entry) => entry.due === null)).toBe(true)
	})

	it('marks the weeks of each window of a windows cadence', () => {
		const cells = buildLineCells(
			contract(),
			line({
				cadence: {
					kind: 'WINDOWS',
					windows: [
						{ from_week: 41, to_week: 42 },
						{ from_week: 45, to_week: 45 },
					],
				},
			}),
			WEEKS,
		)

		expect(cells.filter((entry) => entry.inWindow).map((e) => e.week)).toEqual([
			41, 42, 45,
		])
	})
})

describe('contractTitle', () => {
	it('leads with the job code, then customer and site', () => {
		expect(contractTitle(contract())).toBe('C25029 - GAUME - Jardin')
	})

	it('skips the code and the site when they add nothing', () => {
		expect(
			contractTitle(
				contract({
					reference: null,
					site: { customer_name: 'GAUME', site_label: 'GAUME' },
				}),
			),
		).toBe('GAUME')
	})
})
