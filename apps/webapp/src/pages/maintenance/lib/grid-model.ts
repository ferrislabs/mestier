import { dayOfMonth, isoWeekOf, weekRange } from '#/pages/maintenance/lib/weeks'
import {
	DEFAULT_FIRST_WEEK,
	DEFAULT_LAST_WEEK,
	type GridContract,
	type GridLine,
} from '#/pages/maintenance/types'

/**
 * What a cell shows, in priority order: a visit that happened wins over a due
 * date, which wins over nothing. `out-of-season` is a column the contract does
 * not cover, drawn but never clickable.
 */
export type CellState =
	| 'validated'
	| 'proposed'
	| 'planned'
	| 'due'
	| 'overdue'
	| 'empty'
	| 'out-of-season'

export interface CellVisit {
	id: string
	date: string
	day: number
	status: 'VALIDATED' | 'PROPOSED'
}

export interface CellDue {
	kind: 'planned' | 'due' | 'overdue'
	date: string
	day: number
}

export interface GridCell {
	week: number
	state: CellState
	visits: CellVisit[]
	/** Set even when a visit hides it: a proposed visit does not clear a due date. */
	due: CellDue | null
	/** Inside one of the line's windows, for the windows cadence. */
	inWindow: boolean
	/** A visit can be recorded here: in season, and nothing recorded yet. */
	clickable: boolean
}

const DUE_KIND = {
	PLANNED: 'planned',
	OVERDUE: 'overdue',
	THIS_WEEK: 'due',
	NEXT_WEEK: 'due',
	LATER: 'due',
} as const

/**
 * The columns of the sheet: the widest season among the contracts, so every
 * contract fits, or the paper sheet's S12 to S48 when there is none.
 */
export function sheetWeeks(contracts: GridContract[]): number[] {
	if (contracts.length === 0) {
		return weekRange(DEFAULT_FIRST_WEEK, DEFAULT_LAST_WEEK)
	}
	const first = Math.min(...contracts.map((c) => c.season.first_week))
	const last = Math.max(...contracts.map((c) => c.season.last_week))

	return weekRange(first, last)
}

function windowWeeks(line: GridLine): Set<number> {
	const weeks = new Set<number>()
	if (line.cadence.kind !== 'WINDOWS') return weeks
	for (const window of line.cadence.windows) {
		for (const week of weekRange(window.from_week, window.to_week)) {
			weeks.add(week)
		}
	}

	return weeks
}

function dueOf(
	line: GridLine,
	year: number,
): (CellDue & { week: number }) | null {
	const { on, status } = line.due
	if (!on || status === 'DONE') return null
	const placed = isoWeekOf(on)
	// A date in another ISO year has no column on this year's sheet.
	if (placed.year !== year) return null

	return {
		kind: DUE_KIND[status],
		date: on,
		day: dayOfMonth(on),
		week: placed.week,
	}
}

function stateOf(visits: CellVisit[], due: CellDue | null): CellState {
	const last = visits.at(-1)
	if (last) return last.status === 'VALIDATED' ? 'validated' : 'proposed'

	return due?.kind ?? 'empty'
}

/**
 * One cell per sheet column for one line.
 *
 * The due date is read from the backend (`line.due`), never recomputed: the
 * only work here is finding its column.
 */
export function buildLineCells(
	contract: GridContract,
	line: GridLine,
	weeks: number[],
): GridCell[] {
	const { season } = contract
	const inWindows = windowWeeks(line)
	const due = dueOf(line, season.year)

	const visitsByWeek = new Map<number, CellVisit[]>()
	for (const visit of line.visits) {
		if (visit.status === 'REJECTED') continue
		if (isoWeekOf(visit.done_on).year !== season.year) continue
		const list = visitsByWeek.get(visit.week) ?? []
		list.push({
			id: visit.id,
			date: visit.done_on,
			day: dayOfMonth(visit.done_on),
			status: visit.status === 'VALIDATED' ? 'VALIDATED' : 'PROPOSED',
		})
		visitsByWeek.set(visit.week, list)
	}

	return weeks.map((week) => {
		const inSeason = week >= season.first_week && week <= season.last_week
		const visits = visitsByWeek.get(week) ?? []
		const cellDue = due && due.week === week ? due : null
		if (!inSeason) {
			return {
				week,
				state: 'out-of-season',
				visits,
				due: null,
				inWindow: false,
				clickable: false,
			}
		}

		return {
			week,
			state: stateOf(visits, cellDue),
			visits,
			due: cellDue,
			inWindow: inWindows.has(week),
			clickable: visits.length === 0,
		}
	})
}

/** The heading of a contract block: the job code, then who and where. */
export function contractTitle(contract: GridContract): string {
	const { customer_name, site_label } = contract.site
	const place =
		site_label && site_label !== customer_name
			? `${customer_name} - ${site_label}`
			: customer_name

	return contract.reference ? `${contract.reference} - ${place}` : place
}

/** The `Season` of a contract, `S12 à S48`. */
export function seasonLabel(season: GridContract['season']): string {
	return `S${season.first_week} à S${season.last_week}`
}
