/**
 * ISO-week arithmetic for the season sheet.
 *
 * Everything runs on UTC midnights and `YYYY-MM-DD` strings, so a browser in
 * another time zone, or a clock change, can never move a visit to a
 * neighbouring column. None of this computes a due date: those come from the
 * backend. It only answers "which column does this date belong to".
 */

const DAY_MS = 86_400_000

export interface IsoWeekRef {
	year: number
	week: number
}

function utcMidnight(iso: string): number {
	const [year, month, day] = iso.split('-').map(Number)
	return Date.UTC(year, month - 1, day)
}

function formatUtc(ms: number): string {
	return new Date(ms).toISOString().slice(0, 10)
}

/** Days since Monday, 0 to 6. */
function mondayIndex(ms: number): number {
	return (new Date(ms).getUTCDay() + 6) % 7
}

/** The ISO year and week a `YYYY-MM-DD` date falls in. */
export function isoWeekOf(iso: string): IsoWeekRef {
	const ms = utcMidnight(iso)
	// The week belongs to the year of its Thursday.
	const thursday = ms + (3 - mondayIndex(ms)) * DAY_MS
	const year = new Date(thursday).getUTCFullYear()
	const dayOfYear = Math.round((thursday - Date.UTC(year, 0, 1)) / DAY_MS)

	return { year, week: Math.floor(dayOfYear / 7) + 1 }
}

/** 52 or 53. The 28th of December is always in the last week of its year. */
export function isoWeeksInYear(year: number): number {
	return isoWeekOf(`${year}-12-28`).week
}

/** Monday of an ISO week, as `YYYY-MM-DD`. */
export function mondayOfIsoWeek(year: number, week: number): string {
	// The 4th of January is always in week 1.
	const jan4 = Date.UTC(year, 0, 4)
	const firstMonday = jan4 - mondayIndex(jan4) * DAY_MS

	return formatUtc(firstMonday + (week - 1) * 7 * DAY_MS)
}

/**
 * The date of day-of-month `day` inside an ISO week, or `null` when the week
 * has no such day. A week straddling two months has two spellings of "the
 * 1st", one of which is never typed for it: the week is what disambiguates.
 */
export function dateInIsoWeek(
	year: number,
	week: number,
	day: number,
): string | null {
	const monday = utcMidnight(mondayOfIsoWeek(year, week))
	for (let offset = 0; offset < 7; offset += 1) {
		const candidate = monday + offset * DAY_MS
		if (new Date(candidate).getUTCDate() === day) return formatUtc(candidate)
	}

	return null
}

/** Day of the month of a `YYYY-MM-DD` date: `"2026-10-21"` is `21`. */
export function dayOfMonth(iso: string): number {
	return Number(iso.slice(8, 10))
}

export function weekLabel(week: number): string {
	return `S${week}`
}

/** `from` to `to` inclusive. Empty when the range is reversed. */
export function weekRange(from: number, to: number): number[] {
	const weeks: number[] = []
	for (let week = from; week <= to; week += 1) weeks.push(week)

	return weeks
}
