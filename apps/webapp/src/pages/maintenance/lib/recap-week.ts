import {
	addDays,
	addWeeks,
	format,
	getISOWeek,
	getISOWeekYear,
	startOfISOWeek,
} from 'date-fns'
import { fr } from 'date-fns/locale'
import { z } from 'zod'

const ISO_WEEK = /^(\d{4})-W(\d{1,2})$/

export interface IsoWeek {
	year: number
	week: number
}

/** `2026-W43`, the form the recap endpoints take in `week`. */
export function formatIsoWeek(date: Date): string {
	const week = String(getISOWeek(date)).padStart(2, '0')
	return `${getISOWeekYear(date)}-W${week}`
}

/**
 * The week the recap opens on: always the one after today's.
 *
 * The recap of week W lists what falls due in W and everything due before it
 * that nothing carries yet, so next week's recap already holds whatever is
 * left of this week. That is why the day does not matter.
 */
export function defaultRecapWeek(today: Date): string {
	return formatIsoWeek(addWeeks(today, 1))
}

/** `null` for a malformed value or a week the year does not have (`2027-W53`). */
export function parseIsoWeek(value: string): IsoWeek | null {
	const match = ISO_WEEK.exec(value)
	if (!match) return null
	const year = Number(match[1])
	const week = Number(match[2])
	const monday = mondayOf(year, week)
	if (getISOWeekYear(monday) !== year || getISOWeek(monday) !== week) {
		return null
	}
	return { year, week }
}

export function isoWeekMonday(value: string): Date | null {
	const parsed = parseIsoWeek(value)
	return parsed ? mondayOf(parsed.year, parsed.week) : null
}

/** An unreadable value is returned untouched rather than guessed at. */
export function shiftIsoWeek(value: string, weeks: number): string {
	const monday = isoWeekMonday(value)
	return monday ? formatIsoWeek(addWeeks(monday, weeks)) : value
}

/** `S43 2026 · du 19 oct. au 25 oct.` */
export function weekLabel(value: string): string {
	const parsed = parseIsoWeek(value)
	const monday = isoWeekMonday(value)
	if (!parsed || !monday) return value

	const day = (date: Date) => format(date, 'd MMM', { locale: fr })
	return `S${parsed.week} ${parsed.year} · du ${day(monday)} au ${day(addDays(monday, 6))}`
}

/** January 4th is always in week 1, whatever the weekday it falls on. */
function mondayOf(year: number, week: number): Date {
	return addWeeks(startOfISOWeek(new Date(year, 0, 4)), week - 1)
}

/**
 * `/planification/maintenance/recap?week=2026-W43`. A missing or unreadable
 * week lands on the default one rather than on a router error page.
 */
export const recapSearchSchema = z.object({
	week: z
		.string()
		.refine((value) => parseIsoWeek(value) !== null)
		.catch(() => defaultRecapWeek(new Date())),
})
