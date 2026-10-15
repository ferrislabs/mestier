import { dateInIsoWeek, weekLabel } from '#/pages/maintenance/lib/weeks'

export type VisitDayResult =
	| { ok: true; date: string }
	| { ok: false; error: string }

/**
 * The day typed in a cell, as a date.
 *
 * The sheet only has the day of the month, as on paper; the cell's week says
 * which month it is. A day that week does not contain is refused here rather
 * than guessed. "Not in the future" and "inside the season" stay the
 * backend's call: it answers 409 and the message is shown as is.
 */
export function parseVisitDay(
	input: string,
	year: number,
	week: number,
): VisitDayResult {
	const trimmed = input.trim()
	if (!/^\d{1,2}$/.test(trimmed)) {
		return { ok: false, error: 'Saisissez le jour du mois, par exemple 21.' }
	}
	const date = dateInIsoWeek(year, week, Number(trimmed))
	if (!date) {
		return {
			ok: false,
			error: `Le ${Number(trimmed)} n'est pas dans la semaine ${weekLabel(week)}.`,
		}
	}

	return { ok: true, date }
}
