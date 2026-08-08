/**
 * Whether `task` may have subtasks added under it. The domain caps nesting
 * at two levels — a root and its subtasks, never a subtask's own subtask
 * (see `validate_parent_depth` in `libs/core/src/domain/task/service.rs`,
 * which this mirrors client-side so the third level is never even offered,
 * not merely rejected server-side after the fact).
 */
export function canAddSubtask(task: { parentTaskId: string | null }): boolean {
	return task.parentTaskId === null
}

export interface ResolvedWindow {
	startsAt: string
	endsAt: string
	inherited: boolean
}

/**
 * The window to *show* for a task: its own when it has one, the parent's
 * otherwise — the display-side counterpart of the backend's
 * `resolve_task_window` (see `libs/core/src/domain/task/service.rs`), now
 * that `GET /planning` itself resolves inherited windows before this ever
 * runs. Still useful here for the task form, which needs to know *whether*
 * a window is inherited (to render it as a placeholder rather than a value
 * — see the planning remodel design doc) in a way the read model's already-
 * resolved response doesn't carry.
 */
export function resolveDisplayWindow(
	task: { startsAt: string | null; endsAt: string | null },
	parent: { startsAt: string; endsAt: string } | null,
): ResolvedWindow | null {
	if (task.startsAt && task.endsAt) {
		return { startsAt: task.startsAt, endsAt: task.endsAt, inherited: false }
	}

	if (!parent) return null

	return { startsAt: parent.startsAt, endsAt: parent.endsAt, inherited: true }
}

function formatDateTime(iso: string, timeZone: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		timeZone,
	}).format(new Date(iso))
}

function formatTime(iso: string, timeZone: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		hour: '2-digit',
		minute: '2-digit',
		timeZone,
	}).format(new Date(iso))
}

function sameLocalDay(a: string, b: string, timeZone: string): boolean {
	const format = (iso: string) =>
		new Intl.DateTimeFormat('fr-FR', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
			timeZone,
		}).format(new Date(iso))
	return format(a) === format(b)
}

const ONE_DAY_MS = 24 * 60 * 60 * 1000

/**
 * An all-day window as its day(s) alone — no time-of-day, since none of
 * this is measured in hours. `endsAt` is stored as *exclusive* midnight of
 * the day after the task's last day (the same `[00:00, 24:00)` convention
 * `lib/occurrence.ts` documents for occupying exactly one day, never
 * spilling onto the next), so the displayed last day is `endsAt` minus one
 * day, not `endsAt` itself — showing `endsAt` verbatim would tell a reader
 * the task runs through a day it has already ended before.
 */
function formatAllDayWindow(
	window: { startsAt: string; endsAt: string },
	timeZone: string,
): string {
	const dateOnly = new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		timeZone,
	})
	const start = dateOnly.format(new Date(window.startsAt))
	const lastDay = new Date(new Date(window.endsAt).getTime() - ONE_DAY_MS)
	const end = dateOnly.format(lastDay)

	return start === end ? start : `${start} – ${end}`
}

/**
 * A window as "start – end", the end trimmed to a bare time when it falls
 * on the same local day as the start. The one date-range formatter for the
 * module — {@link formatWindowPlaceholder} and the task list view's rows
 * both build on this rather than re-deriving the same start/end logic.
 *
 * `allDay` switches to {@link formatAllDayWindow} instead: an all-day
 * task's `endsAt` is an exclusive bound (see that function's own doc), so
 * formatting it the same way as a timed window would print a date the task
 * has already finished by. Defaults to `false` — every existing caller
 * (the task form's inherited-window placeholder) keeps its exact prior
 * output.
 */
export function formatWindowRange(
	window: { startsAt: string; endsAt: string },
	timeZone: string,
	options: { allDay?: boolean } = {},
): string {
	if (options.allDay) {
		return formatAllDayWindow(window, timeZone)
	}

	const start = formatDateTime(window.startsAt, timeZone)
	const end = sameLocalDay(window.startsAt, window.endsAt, timeZone)
		? formatTime(window.endsAt, timeZone)
		: formatDateTime(window.endsAt, timeZone)

	return `${start} – ${end}`
}

/**
 * The subtask date field's placeholder text when it is empty — "the window
 * a blank field would inherit", spelled out rather than left for the user
 * to guess (see the planning remodel design doc's "un champ de dates vide
 * qui affiche la fenêtre héritée en placeholder" decision). `null` when
 * there is nothing to inherit (a root task, or a subtask whose parent has
 * not loaded yet).
 */
export function formatWindowPlaceholder(
	window: { startsAt: string; endsAt: string } | null,
	timeZone: string,
): string | null {
	if (!window) return null
	return `Hérite du parent : ${formatWindowRange(window, timeZone)}`
}
