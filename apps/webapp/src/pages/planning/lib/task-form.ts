import { TZDate } from '@date-fns/tz'
import { format } from 'date-fns'
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

function hasOwnDates(values: TaskFormValues): boolean {
	return values.startDate !== '' && values.endDate !== ''
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

/**
 * Mirrors the backend's own checks (`chk_tasks_root_has_dates`,
 * `chk_tasks_dates_both_or_neither`, `chk_tasks_ends_at_after_starts_at`,
 * `chk_tasks_customer_both_or_neither`) ahead of a round-trip — see
 * `libs/core/src/domain/task/service.rs`'s `validate_task_dates`/
 * `validate_customer_pairing`, which this deliberately parallels without
 * importing (this workstream does not own that file).
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

	if (Boolean(values.customerId) !== Boolean(values.customerContextId)) {
		errors.push('Sélectionnez un contexte pour ce client')
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
 * actually displayed. Every existing assignment maps to an `employee`-kind
 * `AssigneeRef`: by the time a `member` assignee reaches `task_assignments`,
 * `TaskService::resolve_assignee` has already provisioned (or found) their
 * `employee_id`, so a `member`-kind ref never survives a round trip.
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
		employee_ids: string[]
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
		assignees: task.employee_ids.map((employeeId) => ({
			kind: 'employee' as const,
			employee_id: employeeId,
		})),
	}
}
