import { TZDate } from '@date-fns/tz'
import { format, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import type { AssigneeRef } from '#/pages/planning/lib/task-drop'

export type CreateTaskRequest = Schemas.CreateTaskRequest
export type UpdateTaskRequest = Schemas.UpdateTaskRequest

/**
 * The task form's own shape — one draft covers both create and edit, root
 * and subtask. `startDate`/`endDate` (and, when not all-day, `startTime`/
 * `endTime`) are strings rather than `Date`s so a blank field is
 * representable: for a subtask, blank means "inherit the parent's window"
 * (see the planning remodel design doc's invariant 8), never a value to
 * coerce. A root task's blank dates are a validation error instead —
 * {@link validateTaskDraft} draws that line, not this shape.
 *
 * `customerId`/`customerContextId` only matter on create — `PATCH
 * /tasks/{id}` has no field for either (see {@link buildPatchTaskPayload}'s
 * own doc), so the edit form never lets them be touched.
 */
export interface TaskFormValues {
	title: string
	description: string
	allDay: boolean
	startDate: string
	startTime: string
	endDate: string
	endTime: string
	/** Asked at creation, never inferred — see the planning remodel design doc's invariant 9. */
	blocksAvailability: boolean
	customerId: string
	customerContextId: string
	labelIds: string[]
	assignees: AssigneeRef[]
}

/**
 * A blank draft. A root task defaults its date fields to `today` (a chantier
 * or meeting has to start somewhere); a subtask defaults them to `''` —
 * inheriting the parent's window is the natural starting point, not an
 * afterthought the user has to discover by clearing the fields themselves.
 */
export function emptyTaskDraft(options: {
	parentTaskId: string | null
	today: string
}): TaskFormValues {
	const isSubtask = options.parentTaskId !== null
	return {
		title: '',
		description: '',
		allDay: false,
		startDate: isSubtask ? '' : options.today,
		startTime: '09:00',
		endDate: isSubtask ? '' : options.today,
		endTime: '10:00',
		blocksAvailability: true,
		customerId: '',
		customerContextId: '',
		labelIds: [],
		assignees: [],
	}
}

export function hasOwnDates(
	values: Pick<TaskFormValues, 'startDate' | 'endDate'>,
): boolean {
	return values.startDate !== '' && values.endDate !== ''
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

// ---------------------------------------------------------------------------
// Date-range + time-of-day pickers (Google Calendar-style) — pure helpers
// backing `ui/task-window-fields.tsx`.
// ---------------------------------------------------------------------------

/** Every pickable time-of-day, on the half-hour — mirrors Google Calendar's own event time dropdown. */
export const TIME_OPTIONS: readonly string[] = Array.from(
	{ length: 48 },
	(_, index) => {
		const totalMinutes = index * 30
		const hours = Math.floor(totalMinutes / 60)
		const minutes = totalMinutes % 60
		return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
	},
)

/**
 * `TIME_OPTIONS` plus `value` itself when it falls off the half-hour grid —
 * a task created before this picker existed (or via a future API caller)
 * may carry an odd minute, and the value must stay visible/selectable
 * rather than silently vanishing from the list.
 */
export function timeOptionsWith(value: string): readonly string[] {
	return TIME_OPTIONS.includes(value)
		? TIME_OPTIONS
		: [...TIME_OPTIONS, value].sort()
}

/** Feeds the `Calendar`'s `mode="range"` `selected` prop from the draft's own `startDate`/`endDate` strings — `undefined` while a subtask inherits (no dates of its own). */
export function dateRangeToCalendarSelection(
	startDate: string,
	endDate: string,
): { from: Date; to: Date } | undefined {
	if (!startDate || !endDate) return undefined
	return { from: parseISO(startDate), to: parseISO(endDate) }
}

/** The inverse of {@link dateRangeToCalendarSelection} — a single from-only click (no `to` yet) folds into a one-day range, mirroring `lib/absences.ts`'s `calendarSelectionToRange`. */
export function calendarSelectionToDateRange(
	selection: { from?: Date; to?: Date } | undefined,
): { startDate: string; endDate: string } | null {
	if (!selection?.from) return null
	const startDate = format(selection.from, 'yyyy-MM-dd')
	const endDate = selection.to ? format(selection.to, 'yyyy-MM-dd') : startDate
	return { startDate, endDate }
}

/** `dd/MM/yyyy`, matching `apps/webapp/src/pages/hr/types.ts`'s own `formatDateFr` — duplicated rather than imported, since this workstream does not own that file. */
function formatDateFr(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(new Date(`${iso}T00:00:00Z`))
}

/** The date-range trigger's label — a single day reads as one date, a real range as `from – to`. */
export function formatDateRangeFr(startDate: string, endDate: string): string {
	return startDate === endDate
		? formatDateFr(startDate)
		: `${formatDateFr(startDate)} – ${formatDateFr(endDate)}`
}

function timeToMinutes(time: string): number {
	const [hours, minutes] = time.split(':').map(Number)
	return hours * 60 + minutes
}

function minutesToTime(totalMinutes: number): string {
	const clamped = Math.max(0, Math.min(23 * 60 + 45, totalMinutes))
	const hours = Math.floor(clamped / 60)
	const minutes = clamped % 60
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

/**
 * Mirrors Google Calendar: editing the start time keeps the event's
 * duration fixed by shifting the end time the same amount. Only for a
 * single-day window (`startDate === endDate`) — a multi-day span's
 * duration isn't a single time-of-day delta, so its end time is left
 * untouched. A shift that would overflow past midnight clamps to the last
 * slot of the day (23:45) rather than rolling into the next day, to avoid
 * silently turning a single-day task into a multi-day one behind the
 * user's back.
 */
export function shiftEndTimeForNewStartTime(
	values: Pick<
		TaskFormValues,
		'startDate' | 'endDate' | 'startTime' | 'endTime'
	>,
	newStartTime: string,
): string {
	if (values.startDate !== values.endDate) return values.endTime

	const durationMinutes =
		timeToMinutes(values.endTime) - timeToMinutes(values.startTime)
	if (durationMinutes <= 0) return values.endTime

	return minutesToTime(timeToMinutes(newStartTime) + durationMinutes)
}

/**
 * Mirrors the backend's own checks (`chk_tasks_root_has_dates`,
 * `chk_tasks_dates_both_or_neither`, `chk_tasks_ends_at_after_starts_at`,
 * `chk_tasks_context_requires_customer`) ahead of a round-trip — see
 * `libs/core/src/domain/task/service.rs`'s `validate_task_dates`/
 * `validate_customer_pairing`, which this deliberately parallels without
 * importing (this workstream does not own that file). A customer with no
 * context is valid (linked to a client, no particular site yet); a context
 * with no customer is not reachable through the form (the context picker
 * only ever renders once a customer is chosen — see `ui/task-form-fields.tsx`)
 * but is still rejected here defensively, mirroring the backend exactly.
 */
export function validateTaskDraft(
	values: TaskFormValues,
	options: { isSubtask: boolean },
): string[] {
	const errors: string[] = []

	if (!values.title.trim()) {
		errors.push('Titre requis')
	}

	const startProvided = values.startDate !== ''
	const endProvided = values.endDate !== ''

	if (startProvided !== endProvided) {
		errors.push('La date de début et la date de fin vont ensemble')
	} else if (!startProvided && !options.isSubtask) {
		errors.push('Une tâche racine doit porter ses propres dates')
	} else if (startProvided && endProvided) {
		if (!values.allDay) {
			if (!isValidTime(values.startTime)) errors.push('Heure de début invalide')
			if (!isValidTime(values.endTime)) errors.push('Heure de fin invalide')
		}
		const startKey = values.allDay
			? values.startDate
			: `${values.startDate}T${values.startTime}`
		const endKey = values.allDay
			? values.endDate
			: `${values.endDate}T${values.endTime}`
		if (values.allDay ? endKey < startKey : endKey <= startKey) {
			errors.push('La fin doit être après le début')
		}
	}

	if (values.customerContextId && !values.customerId) {
		errors.push('Un contexte requiert un client')
	}

	return errors
}

// `TZDate#toISOString` renders with its own zone's offset (correct, but not
// the `Z`-suffixed form the rest of the API traffic uses) — wrapping in a
// plain `Date` before formatting normalizes it without touching the instant.
function toNormalizedIso(zoned: TZDate): string {
	return new Date(zoned.getTime()).toISOString()
}

function addDaysIso(date: string, days: number): string {
	const [year, month, day] = date.split('-').map(Number)
	const base = new Date(Date.UTC(year, month - 1, day))
	base.setUTCDate(base.getUTCDate() + days)
	return base.toISOString().slice(0, 10)
}

/**
 * Resolves the draft's window to `starts_at`/`ends_at`, or `[null, null]`
 * when it has none of its own (a subtask inheriting its parent's — see
 * {@link hasOwnDates}). All-day resolves to local-midnight boundaries, end
 * exclusive — the day after `endDate` — mirroring
 * `apps/webapp/src/pages/hr/lib/absences.ts`'s own half-open convention for
 * the same reason: `ends_at > starts_at` must hold even for a single-day
 * all-day task.
 */
function resolveWindow(
	values: TaskFormValues,
	timeZone: string,
): { starts_at: string | null; ends_at: string | null } {
	if (!hasOwnDates(values)) {
		return { starts_at: null, ends_at: null }
	}

	if (values.allDay) {
		return {
			starts_at: toNormalizedIso(
				new TZDate(`${values.startDate}T00:00:00`, timeZone),
			),
			ends_at: toNormalizedIso(
				new TZDate(`${addDaysIso(values.endDate, 1)}T00:00:00`, timeZone),
			),
		}
	}

	return {
		starts_at: toNormalizedIso(
			new TZDate(`${values.startDate}T${values.startTime}:00`, timeZone),
		),
		ends_at: toNormalizedIso(
			new TZDate(`${values.endDate}T${values.endTime}:00`, timeZone),
		),
	}
}

/**
 * `POST /tasks`'s payload — `null` when the draft doesn't validate. Carries
 * neither `label_ids` nor `assignees`: `CreateTaskRequest` has no field for
 * either (a task is always created bare, per `TaskService::create_task`),
 * so a create that also assigns people or labels needs a follow-up `PATCH`
 * — see {@link needsFollowUpPatch}/{@link buildFollowUpPatchPayload}.
 */
export function buildCreateTaskPayload(
	values: TaskFormValues,
	options: { parentTaskId: string | null; timeZone: string },
): CreateTaskRequest | null {
	const isSubtask = options.parentTaskId !== null
	if (validateTaskDraft(values, { isSubtask }).length > 0) {
		return null
	}

	const window = resolveWindow(values, options.timeZone)

	return {
		title: values.title.trim(),
		description: values.description.trim() || null,
		all_day: values.allDay,
		starts_at: window.starts_at,
		ends_at: window.ends_at,
		blocks_availability: values.blocksAvailability,
		customer_id: values.customerId || null,
		customer_context_id: values.customerContextId || null,
		parent_task_id: options.parentTaskId,
	}
}

/** Whether a freshly created task needs an immediate follow-up `PATCH` to carry the assignees and/or labels picked on the create form. */
export function needsFollowUpPatch(selection: {
	assignees: AssigneeRef[]
	labelIds: string[]
}): boolean {
	return selection.assignees.length > 0 || selection.labelIds.length > 0
}

/** The follow-up `PATCH` a create submit sends right after `POST` succeeds, when {@link needsFollowUpPatch} says one is needed. */
export function buildFollowUpPatchPayload(selection: {
	assignees: AssigneeRef[]
	labelIds: string[]
}): UpdateTaskRequest {
	return {
		assignees: selection.assignees,
		label_ids: selection.labelIds,
	}
}

/**
 * `PATCH /tasks/{id}`'s payload — `null` when the draft doesn't validate.
 * Always resends `assignees` and `label_ids` as complete lists (an empty
 * array included) rather than omitting them when empty: both fields follow
 * the same "complete list, never a delta" contract (see the planning
 * remodel design doc's API section), so there is no "untouched" state to
 * preserve by omission on an edit submit — the form always reflects the
 * task's current selection either way.
 *
 * Never carries `customer_id`/`customer_context_id`: `UpdateTaskRequest`
 * has no field for either, so a task's client is fixed at creation.
 */
export function buildPatchTaskPayload(
	values: TaskFormValues,
	options: { isSubtask: boolean; timeZone: string },
): UpdateTaskRequest | null {
	if (validateTaskDraft(values, { isSubtask: options.isSubtask }).length > 0) {
		return null
	}

	const window = resolveWindow(values, options.timeZone)

	return {
		title: values.title.trim(),
		description: values.description.trim() || null,
		all_day: values.allDay,
		starts_at: window.starts_at,
		ends_at: window.ends_at,
		blocks_availability: values.blocksAvailability,
		label_ids: values.labelIds,
		assignees: values.assignees,
	}
}

/**
 * The inverse of {@link buildPatchTaskPayload} — seeds the edit form from a
 * loaded `TaskResponse`. `customerId`/`customerContextId` are always left
 * blank: they only matter on create (see this file's own doc), so there is
 * nothing to seed the edit form with even when the task has a client — see
 * `ui/task-form-fields.tsx`'s `customerName` prop for how that client is
 * actually displayed.
 */
export function taskToDraft(
	task: {
		title: string
		description?: string | null
		all_day: boolean
		starts_at?: string | null
		ends_at?: string | null
		blocks_availability: boolean
		labels: { id: string }[]
		member_ids: string[]
	},
	timeZone: string,
): TaskFormValues {
	const startZoned = task.starts_at
		? new TZDate(task.starts_at, timeZone)
		: null
	const endZonedRaw = task.ends_at ? new TZDate(task.ends_at, timeZone) : null
	// All-day windows store an exclusive end (the day *after* the last day —
	// see `resolveWindow`'s own doc); this undoes that shift so the field
	// shows the inclusive last day instead.
	const endDate =
		endZonedRaw && task.all_day
			? addDaysIso(format(endZonedRaw, 'yyyy-MM-dd'), -1)
			: endZonedRaw
				? format(endZonedRaw, 'yyyy-MM-dd')
				: ''

	return {
		title: task.title,
		description: task.description ?? '',
		allDay: task.all_day,
		startDate: startZoned ? format(startZoned, 'yyyy-MM-dd') : '',
		startTime: startZoned ? format(startZoned, 'HH:mm') : '09:00',
		endDate,
		endTime: endZonedRaw ? format(endZonedRaw, 'HH:mm') : '10:00',
		blocksAvailability: task.blocks_availability,
		customerId: '',
		customerContextId: '',
		labelIds: task.labels.map((label) => label.id),
		assignees: task.member_ids.map((memberId) => ({
			member_id: memberId,
		})),
	}
}
