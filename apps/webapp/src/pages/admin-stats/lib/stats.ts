import type { Schemas } from '#/api/api.client'

export interface Bar {
	label: string
	count: number
	percent: number
}

export function formatWeekStart(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: 'numeric',
		month: 'short',
	}).format(new Date(`${iso}T00:00:00`))
}

export function weeklyBars(
	weeks: ReadonlyArray<Schemas.WeeklyCountResponse>,
): Bar[] {
	const max = Math.max(0, ...weeks.map((week) => week.count))
	return weeks.map((week) => ({
		label: formatWeekStart(week.week_start),
		count: week.count,
		percent: max === 0 ? 0 : Math.round((week.count / max) * 100),
	}))
}

export function sortedAdoption(
	features: ReadonlyArray<Schemas.FlagAdoptionResponse>,
): Schemas.FlagAdoptionResponse[] {
	return [...features].sort(
		(a, b) =>
			b.organizations - a.organizations || a.label.localeCompare(b.label),
	)
}
