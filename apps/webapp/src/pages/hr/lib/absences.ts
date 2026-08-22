import { TZDate } from '@date-fns/tz'
import { addDays, format, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import {
	toNormalizedIso,
	zonedInstant,
	zonedStartOfDay,
} from '#/lib/zoned-time'

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
 * The inclusive day range the form edits — both ends are calendar days the
 * absence covers, unlike the API's `starts_at`/`ends_at`, which is a
 * half-open window (see {@link AbsenceFormValues}'s own doc). Shaped to
 * match what a shadcn `Calendar` in `mode="range"` hands back through
 * {@link calendarSelectionToRange}, without depending on `react-day-picker`
 * itself — this file stays presentation-library-free.
 */
export interface AbsenceDateRange {
	from: string
	to: string
}

/**
 * The create/edit form's own shape — a date range plus, when it isn't
 * all-day, the time-of-day on each end. `range.to` is the last day
 * *included* (a leave "from Monday to Friday" means five days off), unlike
 * the API's `ends_at`, which is the exclusive end of a half-open window (see
 * the planning design doc's `work_orders`/`absences` decision) —
 * {@link draftToCreateAbsenceRequest} does that conversion.
 */
export interface AbsenceFormValues {
	memberId: string
	kind: AbsenceKind
	allDay: boolean
	range: AbsenceDateRange
	startTime: string
	endTime: string
	note: string
}

/**
 * One row of the member's absence list (`AbsencesSection`) — the draft
 * shape plus the absence's own id. Built via {@link absenceToDraft} so the
 * list shows exactly the same range/allDay/time-of-day resolution the edit
 * form itself uses. `memberId` rides along unused by the list (already
 * scoped to one member, see `EmployeeWorkTimeFeature`) rather than being
 * stripped out — one shape, no destructure-and-discard.
 */
export type AbsenceListItem = AbsenceFormValues & { id: string }

export function emptyAbsenceDraft(
	memberId: string,
	today: string,
): AbsenceFormValues {
	return {
		memberId,
		kind: 'LEAVE',
		allDay: true,
		range: { from: today, to: today },
		startTime: '08:00',
		endTime: '18:00',
		note: '',
	}
}

/**
 * The shadcn `Calendar`'s `mode="range"` selection, translated to
 * {@link AbsenceDateRange}. `to` is `undefined` mid-selection — after the
 * first click, before the second — so it falls back to `from`: a single day
 * is a valid range on its own, which keeps the field fully controlled (no
 * local component state needed in the `ui/` layer to track a pending click,
 * mirroring how `planning-toolbar.tsx`'s single-date `Calendar` is driven
 * straight off props). Returns `null` when nothing is selected (`from`
 * missing), so the caller can leave the current value untouched instead of
 * clearing it.
 */
export function calendarSelectionToRange(
	selection: { from?: Date; to?: Date } | undefined,
): AbsenceDateRange | null {
	if (!selection?.from) return null
	const from = format(selection.from, 'yyyy-MM-dd')
	const to = selection.to ? format(selection.to, 'yyyy-MM-dd') : from
	return { from, to }
}

/** The inverse of {@link calendarSelectionToRange} — feeds the `Calendar`'s `selected` prop from the draft's stored range. */
export function rangeToCalendarSelection(range: AbsenceDateRange): {
	from: Date
	to: Date
} {
	return { from: parseISO(range.from), to: parseISO(range.to) }
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

/**
 * Mirrors the backend's own `CHECK (ends_at > starts_at)` before a
 * round-trip. `requireMember` is `false` for the edit form — the API
 * doesn't accept changing `member_id` on a `PATCH`, so there's nothing to
 * validate there.
 */
export function validateAbsenceDraft(
	draft: AbsenceFormValues,
	options: { requireMember: boolean } = { requireMember: true },
): string[] {
	const errors: string[] = []
	if (options.requireMember && !draft.memberId) {
		errors.push('Personne requise')
	}
	if (!draft.range.from) errors.push('Date de début requise')
	if (!draft.range.to) errors.push('Date de fin requise')
	if (!draft.allDay) {
		if (!isValidTime(draft.startTime)) errors.push('Heure de début invalide')
		if (!isValidTime(draft.endTime)) errors.push('Heure de fin invalide')
	}
	if (draft.range.from && draft.range.to) {
		const startKey = draft.allDay
			? draft.range.from
			: `${draft.range.from}T${draft.startTime}`
		const endKey = draft.allDay
			? draft.range.to
			: `${draft.range.to}T${draft.endTime}`
		if (endKey < startKey) errors.push('La fin doit être après le début')
	}
	return errors
}

function addDaysIso(date: string, days: number): string {
	return format(addDays(parseISO(date), days), 'yyyy-MM-dd')
}

function dateOnlyToIsoMidnight(dateStr: string, timeZone: string): string {
	return toNormalizedIso(zonedStartOfDay(dateStr, timeZone))
}

function dateTimeToIso(
	dateStr: string,
	timeStr: string,
	timeZone: string,
): string {
	return toNormalizedIso(zonedInstant(dateStr, timeStr, timeZone))
}

function buildAbsencePayload(
	draft: AbsenceFormValues,
	timeZone: string,
): Omit<CreateAbsenceRequest, 'member_id'> {
	const startsAt = draft.allDay
		? dateOnlyToIsoMidnight(draft.range.from, timeZone)
		: dateTimeToIso(draft.range.from, draft.startTime, timeZone)
	const endsAt = draft.allDay
		? dateOnlyToIsoMidnight(addDaysIso(draft.range.to, 1), timeZone)
		: dateTimeToIso(draft.range.to, draft.endTime, timeZone)

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
	if (validateAbsenceDraft(draft, { requireMember: true }).length > 0) {
		return null
	}
	return {
		member_id: draft.memberId,
		...buildAbsencePayload(draft, timeZone),
	}
}

/** Same shape as {@link draftToCreateAbsenceRequest}, minus `member_id` — the API doesn't let a `PATCH` reassign an absence to another member. */
export function draftToUpdateAbsenceRequest(
	draft: AbsenceFormValues,
	timeZone: string,
): UpdateAbsenceRequest | null {
	if (validateAbsenceDraft(draft, { requireMember: false }).length > 0) {
		return null
	}
	return buildAbsencePayload(draft, timeZone)
}

/**
 * The inverse of {@link draftToCreateAbsenceRequest} — feeds the edit form
 * from an existing absence. Accepts `absence_kind` rather than `kind`
 * because its usual source is a `PlanningEntry` of kind `absence` (the
 * planning grid's read model disambiguates the entry's own `kind` —
 * `"absence"` — from the absence's `kind` field by renaming the latter); a
 * plain `Schemas.AbsenceResponse` (this module's own list read) maps its
 * `kind` field to `absence_kind` at the call site.
 */
export function absenceToDraft(
	absence: {
		member_id: string
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
			memberId: absence.member_id,
			kind: absence.absence_kind,
			allDay: true,
			range: {
				from: format(startZoned, 'yyyy-MM-dd'),
				to: addDaysIso(endDateExclusive, -1),
			},
			startTime: '08:00',
			endTime: '18:00',
			note: absence.note ?? '',
		}
	}

	return {
		memberId: absence.member_id,
		kind: absence.absence_kind,
		allDay: false,
		range: {
			from: format(startZoned, 'yyyy-MM-dd'),
			to: format(endZoned, 'yyyy-MM-dd'),
		},
		startTime: format(startZoned, 'HH:mm'),
		endTime: format(endZoned, 'HH:mm'),
		note: absence.note ?? '',
	}
}
