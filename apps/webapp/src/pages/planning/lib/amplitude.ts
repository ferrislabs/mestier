import { TZDate } from '@date-fns/tz'
import { format } from 'date-fns'

export const MINUTES_PER_DAY = 24 * 60

export interface MinuteRange {
	startMinute: number
	endMinute: number
}

export const FALLBACK_AMPLITUDE: MinuteRange = {
	startMinute: 8 * 60,
	endMinute: 18 * 60,
}

export interface MinuteInterval {
	startsMinute: number
	endsMinute: number
}

/** A server-side instant span (`starts_at`/`ends_at`), plus its `all_day` flag. */
export interface TimeSpan {
	startsAt: string
	endsAt: string
	allDay: boolean
}

/** Minutes since local midnight, in `timeZone`, for a UTC instant. */
export function minuteOfDayInTimeZone(instant: Date, timeZone: string): number {
	const local = new TZDate(instant, timeZone)
	return local.getHours() * 60 + local.getMinutes()
}

/** The calendar date (`yyyy-MM-dd`), in `timeZone`, for a UTC instant. */
export function isoDateInTimeZone(instant: Date, timeZone: string): string {
	return format(new TZDate(instant, timeZone), 'yyyy-MM-dd')
}

function roundDownToHour(minute: number): number {
	return Math.floor(minute / 60) * 60
}

function roundUpToHour(minute: number): number {
	return Math.ceil(minute / 60) * 60
}

/**
 * The hour range the grid renders, derived once for the whole visible view
 * (see the planning design doc's "amplitude horaire" section) — never per
 * cell, or columns stop lining up. All-day entries carry no time-of-day
 * information — they're stored as the organization's full `[00:00, 24:00)`
 * window (see the domain doc's `tasks`/`absences` decision) —
 * and are excluded here, or a single all-day absence would blow the
 * amplitude open to the full day for everyone.
 */
export function computeAmplitude(
	spans: TimeSpan[],
	workTime: MinuteInterval[],
	timeZone: string,
): MinuteRange {
	const candidates: MinuteRange[] = []

	for (const span of spans) {
		if (span.allDay) continue
		const start = new Date(span.startsAt)
		const end = new Date(span.endsAt)
		const startMinute = minuteOfDayInTimeZone(start, timeZone)
		const sameDay =
			isoDateInTimeZone(start, timeZone) === isoDateInTimeZone(end, timeZone)
		const endMinute = sameDay
			? minuteOfDayInTimeZone(end, timeZone)
			: MINUTES_PER_DAY
		candidates.push({ startMinute, endMinute })
	}

	for (const interval of workTime) {
		candidates.push({
			startMinute: interval.startsMinute,
			endMinute: interval.endsMinute,
		})
	}

	if (candidates.length === 0) return FALLBACK_AMPLITUDE

	const low = Math.min(...candidates.map((candidate) => candidate.startMinute))
	const high = Math.max(...candidates.map((candidate) => candidate.endMinute))
	if (high <= low) return FALLBACK_AMPLITUDE

	return { startMinute: roundDownToHour(low), endMinute: roundUpToHour(high) }
}
