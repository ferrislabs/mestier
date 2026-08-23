import { TZDate } from '@date-fns/tz'
import { format, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import {
	toNormalizedIso,
	zonedInstant,
	zonedStartOfDay,
} from '#/lib/zoned-time'
import { MINUTES_PER_DAY } from '#/pages/planning/lib/amplitude'
import type { AssigneeRef } from '#/pages/planning/lib/task-drop'

export type CreateTaskRequest = Schemas.CreateTaskRequest
export type UpdateTaskRequest = Schemas.UpdateTaskRequest
export type CreateTaskRecurrenceRequest = Schemas.CreateTaskRecurrenceRequest

export type RecurrenceFrequency = 'DAILY' | 'WEEKLY' | 'MONTHLY'

/**
 * The recurrence control's own draft — a sub-shape of {@link TaskFormValues},
 * present on every draft (create or edit) but only ever read on create: an
 * occurrence's `PATCH` never touches the rule (editing one detaches it from
 * the series instead — see the backend's `TaskService::patch_task` doc), and
 * {@link taskToDraft} always seeds it disabled.
 */
export interface RecurrenceDraft {
	enabled: boolean
	frequency: RecurrenceFrequency
	/** ISO weekday numbers, 1 (Monday) through 7 (Sunday). Only read for `WEEKLY`. */
	weekdays: number[]
	/** 1 through 31, clamped to the month's own last day server-side. Only read for `MONTHLY`. */
	dayOfMonth: number
	/** `''` for no end date. */
	endsOn: string
}

export function emptyRecurrenceDraft(): RecurrenceDraft {
	return {
		enabled: false,
		frequency: 'DAILY',
		weekdays: [],
		dayOfMonth: 1,
		endsOn: '',
	}
}

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
	/**
	 * The accepted quote this projet bills against, or `''` for none.
	 *
	 * Picked at creation only, like the customer and for the same reason:
	 * `PATCH /tasks/{id}` carries no `quote_id`. Without it the profitability
	 * report can state a cost but never a margin, since it reads the quote
	 * through this link.
	 */
	quoteId: string
	/**
	 * The project this task is costed against, or `''` for none.
	 *
	 * Unlike the customer and the quote, this *is* editable after creation:
	 * `PATCH /tasks/{id}` carries `project_id`, which is the single way a task
	 * joins or leaves a project.
	 */
	projectId: string
	/**
	 * Expenses in euros as typed, `''` for none. Converted to cents at the
	 * boundary like every other money field in the app.
	 */
	expensesEuros: string
	/** Required as soon as `expensesEuros` is not zero — see {@link validateTaskDraft}. */
	expensesLabel: string
	labelIds: string[]
	assignees: AssigneeRef[]
	/**
	 * Only ever acted on when creating a root task (see
	 * {@link buildCreateTaskRecurrencePayload}) — a subtask cannot repeat
	 * (materialization always produces root tasks), and an occurrence's own
	 * edit form never renders the control at all.
	 */
	recurrence: RecurrenceDraft
}

/**
 * A blank draft. A root task defaults its date fields to `today` (a projet
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
		quoteId: '',
		projectId: '',
		expensesEuros: '',
		expensesLabel: '',
		labelIds: [],
		assignees: [],
		recurrence: emptyRecurrenceDraft(),
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
/**
 * Euros as typed into cents, or `null` when the text is not a number.
 *
 * `''` is zero rather than invalid: an empty expense field means there is
 * nothing to declare, which is the common case.
 */
export function expensesToCents(value: string): number | null {
	const trimmed = value.trim()
	if (trimmed === '') return 0

	const parsed = Number(trimmed.replace(',', '.'))
	if (!Number.isFinite(parsed) || parsed < 0) return null

	return Math.round(parsed * 100)
}

function centsToEuros(cents: number): string {
	return (cents / 100).toFixed(2).replace('.', ',')
}

/**
 * The two expense fields as the API wants them, kept together because they are
 * one value: zero carries no label, and the backend enforces the same pairing.
 */
function expensesPayload(values: TaskFormValues): {
	expenses_cents: number
	expenses_label: string | null
} {
	const cents = expensesToCents(values.expensesEuros) ?? 0

	return {
		expenses_cents: cents,
		expenses_label: cents > 0 ? values.expensesLabel.trim() : null,
	}
}

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

	const expenses = expensesToCents(values.expensesEuros)
	if (expenses === null) {
		errors.push('Montant de frais invalide')
	} else if (expenses > 0 && !values.expensesLabel.trim()) {
		// The API refuses this with a 409, and a form should say so inline rather
		// than let the user discover it on submit: an amount with no reason cannot
		// be audited three months later.
		errors.push('Un montant de frais doit être justifié')
	}

	if (values.recurrence.enabled) {
		if (options.isSubtask) {
			errors.push('Une sous-tâche ne peut pas se répéter')
		}
		if (
			values.recurrence.frequency === 'WEEKLY' &&
			values.recurrence.weekdays.length === 0
		) {
			errors.push('Choisissez au moins un jour de la semaine')
		}
		if (
			values.recurrence.frequency === 'MONTHLY' &&
			(values.recurrence.dayOfMonth < 1 || values.recurrence.dayOfMonth > 31)
		) {
			errors.push('Le jour du mois doit être entre 1 et 31')
		}
		if (values.recurrence.endsOn && values.startDate) {
			if (values.recurrence.endsOn < values.startDate) {
				errors.push('La date de fin de la répétition doit suivre le début')
			}
		}
	}

	return errors
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
			starts_at: toNormalizedIso(zonedStartOfDay(values.startDate, timeZone)),
			ends_at: toNormalizedIso(
				zonedStartOfDay(addDaysIso(values.endDate, 1), timeZone),
			),
		}
	}

	return {
		starts_at: toNormalizedIso(
			zonedInstant(values.startDate, values.startTime, timeZone),
		),
		ends_at: toNormalizedIso(
			zonedInstant(values.endDate, values.endTime, timeZone),
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
		quote_id: values.quoteId || null,
		parent_task_id: options.parentTaskId,
		project_id: values.projectId || null,
		...expensesPayload(values),
	}
}

/**
 * A recurrence's one time-of-day duration, in minutes — every occurrence it
 * materializes shares it (see `CreateTaskRecurrenceCommand::duration_minutes`
 * on the backend). All-day covers the whole calendar day; otherwise it is
 * `endTime - startTime`, floored at one minute so a mistyped equal pair
 * never reaches the API as a non-positive duration (`validateTaskDraft`
 * already refuses `endTime <= startTime` for a same-day window, but a
 * recurrence's own occurrence is always same-day, so this floor is the last
 * line of defense, not the first).
 */
function recurrenceDurationMinutes(values: TaskFormValues): number {
	if (values.allDay) return MINUTES_PER_DAY

	const [startHour, startMinute] = values.startTime.split(':').map(Number)
	const [endHour, endMinute] = values.endTime.split(':').map(Number)
	const start = (startHour ?? 0) * 60 + (startMinute ?? 0)
	const end = (endHour ?? 0) * 60 + (endMinute ?? 0)
	return Math.max(1, end - start)
}

/**
 * `POST /task-recurrences`'s payload — `null` when the draft doesn't
 * validate, is disabled, or has no window of its own (a recurrence always
 * creates root tasks — see `RecurrenceDraft`'s own doc, and `hasOwnDates`
 * for why a subtask never reaches here regardless: `validateTaskDraft`
 * already refuses `recurrence.enabled` on a subtask draft).
 *
 * Carries `assignee_member_ids` directly — unlike {@link buildCreateTaskPayload},
 * there is no follow-up `PATCH`: `CreateTaskRecurrenceCommand` has a field
 * for the template's assignees, applied to every materialized occurrence at
 * creation. It carries no `label_ids`: a label attaches to one task, and
 * there is no "every future occurrence" label endpoint — a label picked on
 * this form would only ever apply to whichever occurrence is edited by hand
 * later, so the control is not offered at all for a recurring draft (see
 * `ui/task-form-fields.tsx`).
 */
export function buildCreateTaskRecurrencePayload(
	values: TaskFormValues,
	options: { timeZone: string },
): CreateTaskRecurrenceRequest | null {
	if (!values.recurrence.enabled) return null
	if (validateTaskDraft(values, { isSubtask: false }).length > 0) return null
	if (!hasOwnDates(values)) return null

	const rule =
		values.recurrence.frequency === 'WEEKLY'
			? { frequency: 'WEEKLY' as const, weekdays: values.recurrence.weekdays }
			: values.recurrence.frequency === 'MONTHLY'
				? {
						frequency: 'MONTHLY' as const,
						day_of_month: values.recurrence.dayOfMonth,
					}
				: { frequency: 'DAILY' as const }

	return {
		...rule,
		starts_on: values.startDate,
		ends_on: values.recurrence.endsOn || null,
		timezone: options.timeZone,
		start_time: values.allDay ? '00:00:00' : `${values.startTime}:00`,
		duration_minutes: recurrenceDurationMinutes(values),
		all_day: values.allDay,
		title: values.title.trim(),
		description: values.description.trim() || null,
		blocks_availability: values.blocksAvailability,
		customer_id: values.customerId || null,
		customer_context_id: values.customerContextId || null,
		project_id: values.projectId || null,
		assignee_member_ids: values.assignees.map((assignee) => assignee.member_id),
	} as CreateTaskRecurrenceRequest
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
		project_id: values.projectId || null,
		...expensesPayload(values),
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
		project_id?: string | null
		/** Absent on the planning read model, which does not carry expenses. */
		expenses_cents?: number
		expenses_label?: string | null
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
		quoteId: '',
		projectId: task.project_id ?? '',
		expensesEuros:
			task.expenses_cents && task.expenses_cents > 0
				? centsToEuros(task.expenses_cents)
				: '',
		expensesLabel: task.expenses_label ?? '',
		labelIds: task.labels.map((label) => label.id),
		assignees: task.member_ids.map((memberId) => ({
			member_id: memberId,
		})),
		// Always disabled: an occurrence's edit form never offers the
		// recurrence control at all (see `RecurrenceDraft`'s own doc) — its
		// series is shown as a notice instead, driven by `task.recurrence_id`
		// directly rather than through this draft.
		recurrence: emptyRecurrenceDraft(),
	}
}
