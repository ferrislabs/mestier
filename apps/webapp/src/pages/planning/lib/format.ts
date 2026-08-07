import type { PlanningView } from '#/pages/planning/types'

function parseUtc(date: string): Date {
	return new Date(`${date}T00:00:00Z`)
}

function capitalize(value: string): string {
	return value.length === 0
		? value
		: value.charAt(0).toUpperCase() + value.slice(1)
}

function formatLongDate(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'long',
		day: 'numeric',
		month: 'long',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(parseUtc(date))
}

function formatMonthYear(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		month: 'long',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(parseUtc(date))
}

function formatShortDate(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: 'numeric',
		month: 'short',
		timeZone: 'UTC',
	}).format(parseUtc(date))
}

/** The toolbar's human-readable label for the visible window, one per granularity. */
export function formatWindowLabel(
	view: PlanningView,
	windowFrom: string,
	windowTo: string,
): string {
	if (view === 'day') return capitalize(formatLongDate(windowFrom))
	if (view === 'month') return capitalize(formatMonthYear(windowFrom))
	return `${formatShortDate(windowFrom)} – ${formatShortDate(windowTo)}`
}
