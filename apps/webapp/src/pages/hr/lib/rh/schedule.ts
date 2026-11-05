import type { DayPart, LunchBreak, WorkDay } from '#/pages/hr/lib/rh/types'

export const DEFAULT_LUNCH: LunchBreak = { start: '12:00', end: '13:00' }

export function minutesOf(time: string): number {
	const [hours, minutes] = time.split(':').map(Number)
	return hours * 60 + minutes
}

export function formatTime(time: string): string {
	const [hours, minutes] = time.split(':')
	return minutes === '00' ? `${Number(hours)}h` : `${Number(hours)}h${minutes}`
}

export function formatSpan(start: string, end: string): string {
	return `${formatTime(start)}–${formatTime(end)}`
}

export function lunchOf(day: Pick<WorkDay, 'lunch'>): LunchBreak | null {
	return day.lunch ?? null
}

export function workDayMinutes(day: WorkDay | null): number {
	if (!day || !day.start || !day.end) return 0
	const span = minutesOf(day.end) - minutesOf(day.start)
	if (span <= 0) return 0
	const lunch = lunchOf(day)
	const pause = lunch ? minutesOf(lunch.end) - minutesOf(lunch.start) : 0
	return Math.max(0, span - pause)
}

export function formatWorkDay(day: WorkDay): string {
	const lunch = lunchOf(day)
	return lunch
		? `${formatSpan(day.start, lunch.start)} · ${formatSpan(lunch.end, day.end)}`
		: formatSpan(day.start, day.end)
}

export function halfRange(
	day: WorkDay,
	part: DayPart,
): { start: string; end: string } {
	const lunch = lunchOf(day)
	if (!lunch)
		return part === 'morning'
			? { start: day.start, end: '12:00' > day.end ? day.end : '12:00' }
			: { start: '12:00' < day.start ? day.start : '12:00', end: day.end }
	return part === 'morning'
		? { start: day.start, end: lunch.start }
		: { start: lunch.end, end: day.end }
}

export function validateWorkDay(day: WorkDay): string | null {
	if (!day.start || !day.end)
		return 'Renseigne l’heure de début et l’heure de fin.'
	if (minutesOf(day.end) <= minutesOf(day.start))
		return 'La fin de journée doit suivre le début.'
	const lunch = lunchOf(day)
	if (!lunch) return null
	if (!lunch.start || !lunch.end)
		return 'Renseigne le début et la fin de la pause.'
	if (minutesOf(lunch.end) <= minutesOf(lunch.start))
		return 'La fin de la pause doit suivre son début.'
	if (
		minutesOf(lunch.start) <= minutesOf(day.start) ||
		minutesOf(lunch.end) >= minutesOf(day.end)
	)
		return 'La pause doit se situer entre le début et la fin de la journée.'
	return null
}
