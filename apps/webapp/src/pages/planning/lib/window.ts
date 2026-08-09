import {
	addDays,
	addMonths,
	addWeeks,
	endOfMonth,
	endOfWeek,
	format,
	parseISO,
	startOfMonth,
	startOfWeek,
} from 'date-fns'
import type { PlanningView } from '#/pages/planning/types'

const ISO_DATE_FORMAT = 'yyyy-MM-dd'
// The organization's work week starts on Monday, not the ambient locale's
// default (which for `en-US` would be Sunday).
const WEEK_STARTS_ON = 1

export interface PlanningWindow {
	from: string
	to: string
}

/**
 * The `[from, to]` window sent to `GET /planning` for a given granularity and
 * anchor date — see the planning design doc's "état de vue dans l'URL"
 * section. Both bounds are inclusive calendar days.
 */
export function computeWindow(
	view: PlanningView,
	date: string,
): PlanningWindow {
	const anchor = parseISO(date)
	switch (view) {
		case 'day':
			return { from: date, to: date }
		case 'week':
			return {
				from: formatIsoDate(
					startOfWeek(anchor, { weekStartsOn: WEEK_STARTS_ON }),
				),
				to: formatIsoDate(endOfWeek(anchor, { weekStartsOn: WEEK_STARTS_ON })),
			}
		case 'month':
			return {
				from: formatIsoDate(startOfMonth(anchor)),
				to: formatIsoDate(endOfMonth(anchor)),
			}
	}
}

/**
 * The window a month **grid** needs: whole weeks, so the first and last rows
 * are complete. Wider than {@link computeWindow}'s month, which stops at the
 * month's own bounds — a grid that stopped there would leave holes at both
 * ends. Kept separate so the team grid keeps asking for exactly its month.
 */
export function computeMonthGridWindow(date: string): PlanningWindow {
	const anchor = parseISO(date)

	return {
		from: formatIsoDate(
			startOfWeek(startOfMonth(anchor), { weekStartsOn: WEEK_STARTS_ON }),
		),
		to: formatIsoDate(
			endOfWeek(endOfMonth(anchor), { weekStartsOn: WEEK_STARTS_ON }),
		),
	}
}

/** Moves the anchor date by one unit of `view` — a day, a week, or a month. */
export function shiftDate(
	view: PlanningView,
	date: string,
	direction: 1 | -1,
): string {
	const anchor = parseISO(date)
	switch (view) {
		case 'day':
			return formatIsoDate(addDays(anchor, direction))
		case 'week':
			return formatIsoDate(addWeeks(anchor, direction))
		case 'month':
			return formatIsoDate(addMonths(anchor, direction))
	}
}

/** Every calendar day in `[from, to]`, inclusive — the column list for week/month views. */
export function enumerateDays(from: string, to: string): string[] {
	const days: string[] = []
	let cursor = parseISO(from)
	const end = parseISO(to)
	while (cursor <= end) {
		days.push(formatIsoDate(cursor))
		cursor = addDays(cursor, 1)
	}
	return days
}

function formatIsoDate(date: Date): string {
	return format(date, ISO_DATE_FORMAT)
}
