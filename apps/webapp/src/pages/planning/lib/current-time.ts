import {
	isoDateInTimeZone,
	type MinuteRange,
	minuteOfDayInTimeZone,
} from '#/pages/planning/lib/amplitude'

const ONE_MINUTE_MS = 60_000

/**
 * Delay, in ms, until the next minute boundary — the anchor for the
 * current-time line's first `setTimeout` (see the planning design doc: an
 * initial `setTimeout` on the remaining seconds, then a one-minute interval,
 * never a raw `setInterval(60_000)`, which drifts). Exactly on a boundary,
 * waits a full minute rather than firing immediately.
 */
export function millisecondsUntilNextMinute(now: Date): number {
	const remainder = now.getTime() % ONE_MINUTE_MS
	return ONE_MINUTE_MS - remainder
}

export interface CurrentTimeVisibilityInput {
	now: Date
	timeZone: string
	windowFrom: string
	windowTo: string
	amplitude: MinuteRange
}

/**
 * The current-time line only makes sense when today is inside the visible
 * window and now falls inside the derived amplitude — outside it, a line
 * pinned to the edge would assert a wrong time (see the planning design
 * doc's "trait de l'heure courante" section).
 */
export function isCurrentTimeVisible(
	input: CurrentTimeVisibilityInput,
): boolean {
	const today = isoDateInTimeZone(input.now, input.timeZone)
	if (today < input.windowFrom || today > input.windowTo) return false

	const minute = minuteOfDayInTimeZone(input.now, input.timeZone)
	return (
		minute >= input.amplitude.startMinute && minute <= input.amplitude.endMinute
	)
}

/** Position along the hour axis (day view), as a percentage of the amplitude. */
export function currentTimePercent(
	now: Date,
	timeZone: string,
	amplitude: MinuteRange,
): number {
	const span = amplitude.endMinute - amplitude.startMinute
	if (span <= 0) return 0
	const minute = minuteOfDayInTimeZone(now, timeZone)
	return ((minute - amplitude.startMinute) / span) * 100
}

/** The index of today's column among `columns` (week/month views), or `null` if absent. */
export function findTodayColumnIndex(
	columns: string[],
	now: Date,
	timeZone: string,
): number | null {
	const today = isoDateInTimeZone(now, timeZone)
	const index = columns.indexOf(today)
	return index === -1 ? null : index
}
