import type { Schemas } from '#/api/api.client'
import { ABSENCE_LABELS } from '#/pages/planning/lib/entries'

export type ConflictResponse = Schemas.ConflictResponse
export type AvailabilityResponse = Schemas.AvailabilityResponse
export type PlanningResourceKind = Schemas.PlanningResourceKindResponse

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
			kind: 'overlapping_work_order'
			workOrderId: string
			startsAt: string
			endsAt: string
	  }
	| { kind: 'missing_employee_record' }

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
		case 'overlapping_work_order':
			return {
				kind: 'overlapping_work_order',
				workOrderId: conflict.work_order_id,
				startsAt: conflict.starts_at,
				endsAt: conflict.ends_at,
			}
	}
}

/**
 * The single `warnings` list a drop's confirmation dialog is fed from — the
 * three natures `GET /planning/availability` reports, plus
 * `missing_employee_record`, deduced client-side from the target resource's
 * `kind` (see the planning design doc's "Avertissements" section). Several
 * natures routinely coexist in the same list — a member-only row on leave
 * produces both an `absence` and a `missing_employee_record` warning — and
 * that's the point: one dialog per gesture, never one per conflict.
 */
export function buildWarnings(params: {
	conflicts: ConflictResponse[]
	resourceKind: PlanningResourceKind
}): Warning[] {
	const warnings = params.conflicts.map(mapConflict)
	if (params.resourceKind === 'member') {
		warnings.push({ kind: 'missing_employee_record' })
	}
	return warnings
}

/** A one-line, human explanation for a warning — feeds the dialog directly. */
export function warningTitle(warning: Warning): string {
	switch (warning.kind) {
		case 'absence':
			return `Absence : ${ABSENCE_LABELS[warning.reason]}`
		case 'outside_work_hours':
			return 'Hors des plages de travail habituelles'
		case 'overlapping_work_order':
			return 'Déjà affecté à un autre chantier sur ce créneau'
		case 'missing_employee_record':
			return 'Aucune fiche employé pour cette personne'
	}
}

/** Secondary detail line, if any — the absence note, or the reminder that a hourly rate will need filling in. */
export function warningDetail(warning: Warning): string | null {
	switch (warning.kind) {
		case 'absence':
			return warning.note?.trim() || null
		case 'missing_employee_record':
			return 'Une fiche employé sera créée automatiquement ; pensez à renseigner son taux horaire ensuite.'
		default:
			return null
	}
}
