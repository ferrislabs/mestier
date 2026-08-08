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

	const start = formatDateTime(window.startsAt, timeZone)
	const end = sameLocalDay(window.startsAt, window.endsAt, timeZone)
		? formatTime(window.endsAt, timeZone)
		: formatDateTime(window.endsAt, timeZone)

	return `Hérite du parent : ${start} – ${end}`
}
