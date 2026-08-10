import type { PlanningEntry } from '#/pages/planning/types'

/**
 * Calendar entry kinds. They reuse the discrimination the API already carries —
 * `kind`, then `absence_kind` for an absence — rather than inventing a parallel
 * taxonomy on the front end.
 */
export type CalendarNature = 'task' | 'leave' | 'sick' | 'unavailable'

export type CalendarFilter = 'all' | CalendarNature

export interface CalendarFilterOption {
	value: CalendarFilter
	label: string
}

export const CALENDAR_FILTER_OPTIONS: CalendarFilterOption[] = [
	{ value: 'all', label: 'Tout' },
	{ value: 'task', label: 'Tâches' },
	{ value: 'leave', label: 'Congés' },
	{ value: 'sick', label: 'Arrêts' },
	{ value: 'unavailable', label: 'Indispo.' },
]

export function entryNature(entry: PlanningEntry): CalendarNature | 'unknown' {
	if (entry.kind === 'task') return 'task'
	if (entry.kind !== 'absence') return 'unknown'

	switch (entry.absence_kind) {
		case 'LEAVE':
			return 'leave'
		case 'SICK':
			return 'sick'
		case 'UNAVAILABLE':
			return 'unavailable'
		default:
			return 'unknown'
	}
}

/**
 * An entry of a kind the front end does not know yet stays visible under
 * "Tout": the calendar degrades instead of silently hiding data the API did
 * return.
 */
export function matchesFilter(
	entry: PlanningEntry,
	filter: CalendarFilter,
): boolean {
	if (filter === 'all') return true
	return entryNature(entry) === filter
}

export function isValidCalendarFilter(value: string): value is CalendarFilter {
	return CALENDAR_FILTER_OPTIONS.some((option) => option.value === value)
}
