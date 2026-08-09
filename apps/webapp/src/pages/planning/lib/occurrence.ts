import { TZDate } from '@date-fns/tz'
import { MINUTES_PER_DAY, type TimeSpan } from '#/pages/planning/lib/amplitude'
import type { MinuteSpan } from '#/pages/planning/lib/layout'

const ONE_DAY_MS = MINUTES_PER_DAY * 60_000

/**
 * The instant of local midnight, in `timeZone`, for calendar day `date`.
 *
 * Built from date parts, not from a string: `new TZDate('2026-08-08T00:00:00',
 * tz)` parses the wall time in the *system* zone and merely displays it in
 * `tz`, so every day boundary was off by the offset between the two whenever
 * they differed — stretching a three-day leave to four in a month grid.
 */
function startOfDayInTimeZone(date: string, timeZone: string): Date {
	const [year, month, day] = date.split('-').map(Number)

	return TZDate.tz(timeZone, year ?? 1970, (month ?? 1) - 1, day ?? 1)
}

/**
 * Whether `span` intersects calendar day `date` in `timeZone`. A half-open
 * interval overlap test, so an all-day entry stored as `[00:00, 24:00)` (see
 * the domain doc) occupies exactly its one day, never spilling onto the next
 * — required for month view's "absences en fond" to shade the right cells.
 */
export function entryOccursOnDate(
	span: TimeSpan,
	date: string,
	timeZone: string,
): boolean {
	const dayStart = startOfDayInTimeZone(date, timeZone).getTime()
	const dayEnd = dayStart + ONE_DAY_MS
	const start = new Date(span.startsAt).getTime()
	const end = new Date(span.endsAt).getTime()
	return start < dayEnd && end > dayStart
}

/**
 * The portion of `span` that falls on calendar day `date`, as minutes since
 * that day's local midnight — clipped to `[0, 1440]` for an entry that
 * starts before or ends after the day. Feeds directly into
 * {@link import('#/pages/planning/lib/layout').computeSegmentPosition}
 * against the shared amplitude, for day, week and month cells alike.
 */
export function minuteSpanOnDate(
	span: TimeSpan,
	date: string,
	timeZone: string,
): MinuteSpan {
	if (span.allDay) return { startMinute: 0, endMinute: MINUTES_PER_DAY }

	const dayStart = startOfDayInTimeZone(date, timeZone).getTime()
	const dayEnd = dayStart + ONE_DAY_MS
	const start = Math.max(new Date(span.startsAt).getTime(), dayStart)
	const end = Math.min(new Date(span.endsAt).getTime(), dayEnd)

	return {
		startMinute: Math.round((start - dayStart) / 60_000),
		endMinute: Math.round((end - dayStart) / 60_000),
	}
}
