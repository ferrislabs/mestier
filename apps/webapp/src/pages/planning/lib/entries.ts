import type { Schemas } from '#/api/api.client'
import type { PlanningEntry } from '#/pages/planning/types'

export const ABSENCE_LABELS: Record<Schemas.AbsenceKind, string> = {
	LEAVE: 'Congé',
	SICK: 'Arrêt maladie',
	UNAVAILABLE: 'Indisponible',
}

/**
 * The member(s) an entry appears against. `task` entries can carry several
 * assignees; `absence` entries exactly one. `PlanningEntryResponse` is a
 * discriminated union that will grow a third, external-source variant later
 * (see the planning design doc) — an entry outside today's two known kinds
 * resolves to no resource rather than throwing, so the grid degrades instead
 * of crashing on data it doesn't understand yet.
 */
export function entryMemberIds(entry: PlanningEntry): string[] {
	if (entry.kind === 'task') return entry.member_ids
	if (entry.kind === 'absence') return [entry.member_id]
	return []
}

/** A human label for the entry's segment — title for a task, motive for an absence. `title` is required on a task, so no fallback is needed. */
export function entryLabel(entry: PlanningEntry): string {
	if (entry.kind === 'task') return entry.title
	if (entry.kind === 'absence')
		return ABSENCE_LABELS[entry.absence_kind] ?? 'Absence'
	return 'Entrée'
}

/**
 * Whether `entry` follows a recurrence — what the calendar, team grid and
 * month view all mark with a repeat icon (see `mestier_core::Task::recurrence_id`'s
 * own doc: `null` covers both "never part of a series" and "detached from
 * one by an edit", and either way there is nothing to mark). Only a `task`
 * entry can carry one; an absence never does.
 */
export function entryIsRecurring(entry: PlanningEntry): boolean {
	return entry.kind === 'task' && entry.recurrence_id != null
}

export type EntryTone = 'task' | 'absence' | 'unknown'

/** Drives the segment's styling — see {@link entryMemberIds} on the unknown-kind fallback. */
export function entryTone(entry: PlanningEntry): EntryTone {
	if (entry.kind === 'task') return 'task'
	if (entry.kind === 'absence') return 'absence'
	return 'unknown'
}
