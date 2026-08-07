import { TZDate } from '@date-fns/tz'
import { addDays, format, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'

export type AbsenceKind = Schemas.AbsenceKind
export type CreateAbsenceRequest = Schemas.CreateAbsenceRequest
export type UpdateAbsenceRequest = Schemas.UpdateAbsenceRequest

export const ABSENCE_KINDS: AbsenceKind[] = ['LEAVE', 'SICK', 'UNAVAILABLE']

export const ABSENCE_KIND_LABELS: Record<AbsenceKind, string> = {
	LEAVE: 'Congé',
	SICK: 'Arrêt maladie',
	UNAVAILABLE: 'Indisponible',
}

/**
 * The create/edit form's own shape — a date range plus, when it isn't
 * all-day, the time-of-day on each end. `endDate` is the last day *included*
 * (a leave "from Monday to Friday" means five days off), unlike the API's
 * `ends_at`, which is the exclusive end of a half-open window (see the
 * planning design doc's `work_orders`/`employee_absences` decision) —
 * {@link draftToCreateAbsenceRequest} does that conversion.
 */
export interface AbsenceFormValues {
	employeeId: string
	kind: AbsenceKind
	allDay: boolean
	startDate: string
	startTime: string
	endDate: string
	endTime: string
	note: string
}

export function emptyAbsenceDraft(
	employeeId: string,
	today: string,
): AbsenceFormValues {
	return {
		employeeId,
		kind: 'LEAVE',
		allDay: true,
		startDate: today,
		startTime: '08:00',
		endDate: today,
		endTime: '18:00',
		note: '',
	}
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

/**
 * Mirrors the backend's own `CHECK (ends_at > starts_at)` before a
 * round-trip. `requireEmployee` is `false` for the edit form — the API
 * doesn't accept changing `employee_id` on a `PATCH`, so there's nothing to
 * validate there.
 */
export function validateAbsenceDraft(
	draft: AbsenceFormValues,
	options: { requireEmployee: boolean } = { requireEmployee: true },
): string[] {
	const errors: string[] = []
	if (options.requireEmployee && !draft.employeeId) {
		errors.push('Employé requis')
	}
	if (!draft.startDate) errors.push('Date de début requise')
	if (!draft.endDate) errors.push('Date de fin requise')
	if (!draft.allDay) {
		if (!isValidTime(draft.startTime)) errors.push('Heure de début invalide')
		if (!isValidTime(draft.endTime)) errors.push('Heure de fin invalide')
	}
	if (draft.startDate && draft.endDate) {
		const startKey = draft.allDay
			? draft.startDate
			: `${draft.startDate}T${draft.startTime}`
		const endKey = draft.allDay
			? draft.endDate
			: `${draft.endDate}T${draft.endTime}`
		if (endKey < startKey) errors.push('La fin doit être après le début')
	}
	return errors
}

function addDaysIso(date: string, days: number): string {
	return format(addDays(parseISO(date), days), 'yyyy-MM-dd')
}

// `TZDate#toISOString` renders with its own zone's offset (correct, but not
// the `Z`-suffixed form the rest of the API traffic uses) — wrapping in a
// plain `Date` before formatting normalizes it without touching the instant.
function toNormalizedIso(zoned: TZDate): string {
	return new Date(zoned.getTime()).toISOString()
}

function dateOnlyToIsoMidnight(dateStr: string, timeZone: string): string {
	return toNormalizedIso(new TZDate(`${dateStr}T00:00:00`, timeZone))
}

function dateTimeToIso(
	dateStr: string,
	timeStr: string,
	timeZone: string,
): string {
	return toNormalizedIso(new TZDate(`${dateStr}T${timeStr}:00`, timeZone))
}

function buildAbsencePayload(
	draft: AbsenceFormValues,
	timeZone: string,
): Omit<CreateAbsenceRequest, 'employee_id'> {
	const startsAt = draft.allDay
		? dateOnlyToIsoMidnight(draft.startDate, timeZone)
		: dateTimeToIso(draft.startDate, draft.startTime, timeZone)
	const endsAt = draft.allDay
		? dateOnlyToIsoMidnight(addDaysIso(draft.endDate, 1), timeZone)
		: dateTimeToIso(draft.endDate, draft.endTime, timeZone)

	return {
		kind: draft.kind,
		all_day: draft.allDay,
		starts_at: startsAt,
		ends_at: endsAt,
		note: draft.note.trim() || null,
	}
}

/**
 * `starts_at`/`ends_at` for a create request, or `null` when the draft
 * doesn't validate — the caller shouldn't fabricate a payload out of an
 * invalid form rather than checking first.
 */
export function draftToCreateAbsenceRequest(
	draft: AbsenceFormValues,
	timeZone: string,
): CreateAbsenceRequest | null {
	if (validateAbsenceDraft(draft, { requireEmployee: true }).length > 0) {
		return null
	}
	return {
		employee_id: draft.employeeId,
		...buildAbsencePayload(draft, timeZone),
	}
}

/** Same shape as {@link draftToCreateAbsenceRequest}, minus `employee_id` — the API doesn't let a `PATCH` reassign an absence to another employee. */
export function draftToUpdateAbsenceRequest(
	draft: AbsenceFormValues,
	timeZone: string,
): UpdateAbsenceRequest | null {
	if (validateAbsenceDraft(draft, { requireEmployee: false }).length > 0) {
		return null
	}
	return buildAbsencePayload(draft, timeZone)
}

/** The inverse of {@link draftToCreateAbsenceRequest} — feeds the edit form from an existing absence (or the matching `PlanningEntry` — same fields). */
export function absenceToDraft(
	absence: {
		employee_id: string
		absence_kind: AbsenceKind
		all_day: boolean
		starts_at: string
		ends_at: string
		note?: string | null
	},
	timeZone: string,
): AbsenceFormValues {
	const startZoned = new TZDate(absence.starts_at, timeZone)
	const endZoned = new TZDate(absence.ends_at, timeZone)

	if (absence.all_day) {
		const endDateExclusive = format(endZoned, 'yyyy-MM-dd')
		return {
			employeeId: absence.employee_id,
			kind: absence.absence_kind,
			allDay: true,
			startDate: format(startZoned, 'yyyy-MM-dd'),
			startTime: '08:00',
			endDate: addDaysIso(endDateExclusive, -1),
			endTime: '18:00',
			note: absence.note ?? '',
		}
	}

	return {
		employeeId: absence.employee_id,
		kind: absence.absence_kind,
		allDay: false,
		startDate: format(startZoned, 'yyyy-MM-dd'),
		startTime: format(startZoned, 'HH:mm'),
		endDate: format(endZoned, 'yyyy-MM-dd'),
		endTime: format(endZoned, 'HH:mm'),
		note: absence.note ?? '',
	}
}
