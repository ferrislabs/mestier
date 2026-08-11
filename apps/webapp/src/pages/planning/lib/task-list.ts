import type { Schemas } from '#/api/api.client'

export type PaginationMetadata = Schemas.PaginationMetadata

/**
 * Whether a root task's row should render an expand/collapse chevron.
 * `GET /tasks` computes `child_count` server-side without loading the
 * hierarchy (see the planning remodel design doc's API section) — a root
 * with zero children never offers to expand, rather than expanding onto an
 * empty list.
 */
export function taskHasChildren(task: {
	child_count?: number | null
}): boolean {
	return (task.child_count ?? 0) > 0
}

/** Whether a next page exists — `null`/`undefined` metadata (not loaded yet) means no. */
export function canGoToNextPage(
	pagination: PaginationMetadata | null | undefined,
): boolean {
	return pagination?.next_page != null
}

/** Whether a previous page exists. */
export function canGoToPreviousPage(
	pagination: PaginationMetadata | null | undefined,
): boolean {
	return pagination?.prev_page != null
}

/** Adds `taskId` to the expanded set if absent, removes it if present — never mutates `expanded`. */
export function toggleExpandedTask(
	expanded: readonly string[],
	taskId: string,
): string[] {
	return expanded.includes(taskId)
		? expanded.filter((id) => id !== taskId)
		: [...expanded, taskId]
}

interface ResourceLike {
	member_id: string
	display_name: string
}

/**
 * Member id → display name, built from `GET /planning`'s resource roster —
 * see `feature/task-list-feature.tsx`'s own doc on why the list view
 * piggybacks on that endpoint rather than a second member fetch.
 */
export function memberNamesById(
	resources: ResourceLike[],
): Record<string, string> {
	const names: Record<string, string> = {}
	for (const resource of resources) {
		names[resource.member_id] = resource.display_name
	}
	return names
}

/**
 * Resolves a task's assignee ids to display names for row rendering. An id
 * absent from `namesById` (the roster hasn't loaded yet, or the member was
 * removed after assignment) falls back to a placeholder rather than
 * silently dropping the assignee from the count.
 */
export function resolveAssigneeNames(
	memberIds: string[],
	namesById: Record<string, string>,
): string[] {
	return memberIds.map((id) => namesById[id] ?? 'Assigné inconnu')
}

/** The row's assignee cell text — never empty, so an unassigned task still reads as a deliberate state rather than a blank cell. */
export function formatAssigneeNames(names: string[]): string {
	return names.length === 0 ? 'Personne assigné' : names.join(', ')
}

/**
 * Which task's `all_day` flag governs a subtask row's window display.
 * `all_day` lives on the task actually being shown — the subtask's own
 * when it has its own dates, the root's when the row is showing the
 * root's window because the subtask has none (see
 * `lib/subtasks.ts`'s `resolveDisplayWindow`'s `inherited` flag). A
 * subtask can carry `all_day: true` on itself while having no dates of
 * its own; that flag is meaningless while the row is displaying the
 * root's timed window instead.
 */
export function resolveSubtaskAllDay(
	inherited: boolean,
	subtaskAllDay: boolean,
	rootAllDay: boolean,
): boolean {
	return inherited ? rootAllDay : subtaskAllDay
}
