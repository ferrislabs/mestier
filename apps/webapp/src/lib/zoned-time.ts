import { TZDate } from '@date-fns/tz'

/**
 * Turning a wall-clock date and time into an instant, in a named time zone.
 *
 * `new TZDate('2026-08-10T09:00:00', tz)` does **not** do this. It parses the
 * string with `Date` semantics — the *system* zone — and then merely displays
 * the result in `tz`. The same call therefore yields a different instant on
 * every machine:
 *
 * ```text
 * ambient Europe/Paris  ->  2026-08-10T07:00:00Z   (what the organization meant)
 * ambient UTC           ->  2026-08-10T09:00:00Z
 * ambient Asia/Tokyo    ->  2026-08-10T00:00:00Z
 * ```
 *
 * Which means every task window and every absence was stored correctly only for
 * users whose browser happened to sit in the organization's zone. `TZDate.tz`
 * takes the parts and interprets them in the zone, so the answer is the same
 * everywhere.
 *
 * `planning/lib/occurrence.ts` had already worked this out and fixed it for the
 * month grid — a three-day leave used to stretch to four. It kept its own
 * private helper, so the same trap survived in the task form and in the absence
 * form. This module exists so there is one of them.
 */

/** The instant of local midnight in `timeZone`, for calendar day `date`. */
export function zonedStartOfDay(date: string, timeZone: string): Date {
	const [year, month, day] = date.split('-').map(Number)

	return TZDate.tz(timeZone, year ?? 1970, (month ?? 1) - 1, day ?? 1)
}

/**
 * The instant of `date` at `time` (`HH:mm`) in `timeZone`.
 *
 * A malformed `time` falls back to midnight rather than throwing: callers
 * validate the field before submitting, and a crash inside a date helper would
 * be a worse way to learn about it.
 */
export function zonedInstant(
	date: string,
	time: string,
	timeZone: string,
): Date {
	const [year, month, day] = date.split('-').map(Number)
	const [hour, minute] = time.split(':').map(Number)

	return TZDate.tz(
		timeZone,
		year ?? 1970,
		(month ?? 1) - 1,
		day ?? 1,
		hour ?? 0,
		minute ?? 0,
	)
}

/**
 * `Z`-suffixed ISO, the form the rest of the API traffic uses.
 *
 * `TZDate#toISOString` renders with its own zone's offset — correct, but not
 * that shape. Wrapping in a plain `Date` normalizes it without moving the
 * instant.
 */
export function toNormalizedIso(instant: Date): string {
	return new Date(instant.getTime()).toISOString()
}
