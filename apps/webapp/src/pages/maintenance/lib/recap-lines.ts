import { format, parseISO } from 'date-fns'
import { fr } from 'date-fns/locale'
import type { Schemas } from '#/api/api.client'
import type { MaintenanceRecapGroup } from '#/hooks/use-maintenance-recap'

export function lineIdsOf(groups: MaintenanceRecapGroup[]): string[] {
	return groups.flatMap((group) => group.lines.map((line) => line.line_id))
}

/**
 * The selection restricted to the lines still on screen. Planning lines
 * removes them from the recap; a selection that still named them would make
 * the next "Planifier" send ids the server no longer lists.
 */
export function pruneSelection(
	selection: ReadonlySet<string>,
	groups: MaintenanceRecapGroup[],
): ReadonlySet<string> {
	const present = new Set(lineIdsOf(groups))
	const kept = [...selection].filter((id) => present.has(id))
	return kept.length === selection.size ? selection : new Set(kept)
}

export function townLabel(town: string | null | undefined): string {
	return town ? town : 'Sans commune'
}

export interface DueBadge {
	label: string
	tone: 'error' | 'warning' | 'brand' | 'neutral' | 'success'
}

/**
 * The recap computes every due date at the Monday of its week, so
 * `THIS_WEEK` means "in the week being read", whichever week that is.
 */
export function dueBadge(status: Schemas.DueStatus): DueBadge {
	switch (status) {
		case 'OVERDUE':
			return { label: 'En retard', tone: 'error' }
		case 'THIS_WEEK':
			return { label: 'Dans la semaine', tone: 'warning' }
		case 'NEXT_WEEK':
			return { label: 'Semaine suivante', tone: 'brand' }
		case 'LATER':
			return { label: 'Plus tard', tone: 'neutral' }
		case 'PLANNED':
			return { label: 'Planifiée', tone: 'neutral' }
		case 'DONE':
			return { label: 'Terminée', tone: 'success' }
	}
}

/** `1 h 30`, `2 h`, `45 min`. */
export function hoursLabel(minutes: number | null | undefined): string {
	if (minutes === null || minutes === undefined) return '—'
	const hours = Math.floor(minutes / 60)
	const rest = minutes % 60
	if (hours === 0) return `${minutes} min`
	return rest === 0
		? `${hours} h`
		: `${hours} h ${String(rest).padStart(2, '0')}`
}

/** `19 oct.`, from a `YYYY-MM-DD` day. */
export function formatShortDate(day: string | null | undefined): string {
	if (!day) return '—'
	return format(parseISO(day), 'd MMM', { locale: fr })
}

export function lineSiteLabel(site: {
	customer_name: string
	site_label: string
	city?: string | null | undefined
}): string {
	const base = `${site.customer_name} · ${site.site_label}`
	return site.city ? `${base} (${site.city})` : base
}
