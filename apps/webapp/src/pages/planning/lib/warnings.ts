import type { Schemas } from '#/api/api.client'
import { ABSENCE_LABELS } from '#/pages/planning/lib/entries'

export type ConflictResponse = Schemas.ConflictResponse
export type AvailabilityResponse = Schemas.AvailabilityResponse

export type Warning =
	| {
			kind: 'absence'
			reason: Schemas.AbsenceKind
			note: string | null
			startsAt: string
			endsAt: string
	  }
	| { kind: 'outside_work_hours'; startsAt: string; endsAt: string }
	| {
			kind: 'overlapping_task'
			taskId: string
			startsAt: string
			endsAt: string
	  }

/**
 * The conflicts `GET /planning/availability` reports for a single resource —
 * `[]` when the resource doesn't appear in the response (no conflict is a
 * legitimate reason to be absent from it, not an error).
 */
export function conflictsForResource(
	availability: AvailabilityResponse,
	resourceId: string,
): ConflictResponse[] {
	return (
		availability.resources.find(
			(resource) => resource.resource_id === resourceId,
		)?.conflicts ?? []
	)
}

function mapConflict(conflict: ConflictResponse): Warning {
	switch (conflict.kind) {
		case 'absence':
			return {
				kind: 'absence',
				reason: conflict.reason,
				note: conflict.note ?? null,
				startsAt: conflict.starts_at,
				endsAt: conflict.ends_at,
			}
		case 'outside_work_hours':
			return {
				kind: 'outside_work_hours',
				startsAt: conflict.starts_at,
				endsAt: conflict.ends_at,
			}
		case 'overlapping_task':
			return {
				kind: 'overlapping_task',
				taskId: conflict.task_id,
				startsAt: conflict.starts_at,
				endsAt: conflict.ends_at,
			}
	}
}

/**
 * The single `warnings` list a drop's confirmation dialog is fed from — the
 * three natures `GET /planning/availability` reports (see the planning
 * design doc's "Avertissements" section). Several natures routinely coexist
 * in the same list, which is the point: one dialog per gesture, never one
 * per conflict.
 */
export function buildWarnings(params: {
	conflicts: ConflictResponse[]
}): Warning[] {
	return params.conflicts.map(mapConflict)
}

/** A one-line, human explanation for a warning — feeds the dialog directly. */
export function warningTitle(warning: Warning): string {
	switch (warning.kind) {
		case 'absence':
			return `Absence : ${ABSENCE_LABELS[warning.reason]}`
		case 'outside_work_hours':
			return 'Hors des plages de travail habituelles'
		case 'overlapping_task':
			return 'Déjà affecté à un autre chantier sur ce créneau'
	}
}

/** Secondary detail line, if any — the absence note. */
export function warningDetail(warning: Warning): string | null {
	switch (warning.kind) {
		case 'absence':
			return warning.note?.trim() || null
		default:
			return null
	}
}
