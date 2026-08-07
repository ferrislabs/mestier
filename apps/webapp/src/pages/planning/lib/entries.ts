import type { Schemas } from '#/api/api.client'
import type { PlanningEntry } from '#/pages/planning/types'

const ABSENCE_LABELS: Record<Schemas.AbsenceKind, string> = {
	LEAVE: 'Congé',
	SICK: 'Arrêt maladie',
	UNAVAILABLE: 'Indisponible',
}

/**
 * The employee(s) an entry appears against. `work_order` entries can carry
 * several assignees; `absence` entries exactly one. `PlanningEntryResponse`
 * is a discriminated union that will grow a third, external-source variant
 * later (see the planning design doc) — an entry outside today's two known
 * kinds resolves to no resource rather than throwing, so the grid degrades
 * instead of crashing on data it doesn't understand yet.
 */
export function entryEmployeeIds(entry: PlanningEntry): string[] {
	if (entry.kind === 'work_order') return entry.employee_ids
	if (entry.kind === 'absence') return [entry.employee_id]
	return []
}

/** A human label for the entry's segment — title/customer for a work order, motive for an absence. */
export function entryLabel(entry: PlanningEntry): string {
	if (entry.kind === 'work_order')
		return entry.title?.trim() || entry.customer_name
	if (entry.kind === 'absence')
		return ABSENCE_LABELS[entry.absence_kind] ?? 'Absence'
	return 'Entrée'
}

export type EntryTone = 'work_order' | 'absence' | 'unknown'

/** Drives the segment's styling — see {@link entryEmployeeIds} on the unknown-kind fallback. */
export function entryTone(entry: PlanningEntry): EntryTone {
	if (entry.kind === 'work_order') return 'work_order'
	if (entry.kind === 'absence') return 'absence'
	return 'unknown'
}
