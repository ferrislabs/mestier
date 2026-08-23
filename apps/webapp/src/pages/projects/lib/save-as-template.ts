import { TZDate } from '@date-fns/tz'
import { differenceInCalendarDays, getHours, getMinutes } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import {
	emptyTemplateTaskDraft,
	type TemplateTaskDraft,
} from '#/pages/project-templates/lib/template-task-form'

type Task = Schemas.TaskResponse

/**
 * Resolves a task's own window, falling back to its root's when it has none
 * of its own (a subtask inheriting its parent's window — see
 * `resolve_task_window` on the backend). `roots` is keyed by id so this
 * stays a lookup rather than a scan per task.
 */
function resolvedWindow(
	task: Task,
	roots: Map<string, Task>,
): { starts_at: string; ends_at: string } | null {
	if (task.starts_at && task.ends_at) {
		return { starts_at: task.starts_at, ends_at: task.ends_at }
	}
	if (!task.parent_task_id) return null
	const root = roots.get(task.parent_task_id)
	if (!root?.starts_at || !root.ends_at) return null
	return { starts_at: root.starts_at, ends_at: root.ends_at }
}

/**
 * Converts a project's tasks (roots and their subtasks) into template task
 * shapes: absolute dates become offsets from the earliest task, and
 * assignees are dropped entirely — a template never carries one (see
 * `#/pages/project-templates/lib/template-task-form`'s own doc on why).
 *
 * Order is preserved (roots first, in their own array order, each followed
 * by nothing — subtasks are appended after every root so every
 * `parent_index` can point backwards), and a subtask whose root did not
 * make it into `tasks` (should not happen — a project's task list always
 * includes both) is dropped rather than crashing the export.
 */
export function projectTasksToTemplateDrafts(
	tasks: Task[],
	timeZone: string,
): TemplateTaskDraft[] {
	const roots = tasks.filter((task) => !task.parent_task_id)
	const children = tasks.filter((task) => task.parent_task_id)
	const rootsById = new Map(roots.map((root) => [root.id, root]))

	const resolved = [...roots, ...children]
		.map((task) => ({ task, window: resolvedWindow(task, rootsById) }))
		.filter(
			(
				entry,
			): entry is { task: Task; window: NonNullable<typeof entry.window> } =>
				entry.window !== null,
		)

	if (resolved.length === 0) return []

	const anchor = resolved.reduce((earliest, entry) =>
		entry.window.starts_at < earliest.window.starts_at ? entry : earliest,
	).window.starts_at
	const anchorDate = new TZDate(anchor, timeZone)

	const orderedIds = resolved.map((entry) => entry.task.id)
	const indexById = new Map(orderedIds.map((id, index) => [id, index]))

	return resolved.map(({ task, window }) => {
		const draft: TemplateTaskDraft = emptyTemplateTaskDraft()
		const startZoned = new TZDate(window.starts_at, timeZone)
		const endZoned = new TZDate(window.ends_at, timeZone)

		draft.title = task.title
		draft.description = task.description ?? ''
		draft.dayOffset = differenceInCalendarDays(startZoned, anchorDate)
		draft.allDay = task.all_day
		draft.blocksAvailability = task.blocks_availability
		draft.expensesEuros =
			task.expenses_cents > 0
				? (task.expenses_cents / 100).toFixed(2).replace('.', ',')
				: ''
		draft.expensesLabel = task.expenses_label ?? ''
		draft.parentIndex = task.parent_task_id
			? (indexById.get(task.parent_task_id) ?? null)
			: null

		if (!task.all_day) {
			draft.startTime = `${String(getHours(startZoned)).padStart(2, '0')}:${String(getMinutes(startZoned)).padStart(2, '0')}`
			draft.endTime = `${String(getHours(endZoned)).padStart(2, '0')}:${String(getMinutes(endZoned)).padStart(2, '0')}`
		}

		return draft
	})
}
