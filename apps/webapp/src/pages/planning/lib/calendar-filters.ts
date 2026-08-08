import type { PlanningEntry } from '#/pages/planning/types'

/**
 * Natures d'entrée du calendrier. Elles reprennent la discrimination déjà
 * portée par l'API — `kind` puis, pour une absence, `absence_kind` — plutôt
 * que d'inventer une taxonomie parallèle côté front.
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
 * Une entrée d'une nature que le front ne connaît pas encore reste visible
 * sous « Tout » : le calendrier dégrade au lieu de masquer silencieusement une
 * donnée que l'API a bien renvoyée.
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
