interface ResourceLike {
	member_id: string
	display_name: string
}

/**
 * Member id → display name, built from `GET /planning`'s resource roster.
 *
 * Lived in `lib/task-list.ts` until the board replaced the task list (#466);
 * it outlived that file because two screens still need it — the assignment
 * report list and the board's cards — and neither wants the pagination and
 * expand/collapse helpers it used to sit beside.
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
 * Resolves a task's assignee ids to display names. An id absent from
 * `namesById` (the roster hasn't loaded yet, or the member was removed after
 * assignment) falls back to a placeholder rather than silently dropping the
 * assignee from the count.
 */
export function resolveAssigneeNames(
	memberIds: string[],
	namesById: Record<string, string>,
): string[] {
	return memberIds.map((id) => namesById[id] ?? 'Assigné inconnu')
}

/** The assignee text — never empty, so an unassigned task still reads as a deliberate state rather than a blank. */
export function formatAssigneeNames(names: string[]): string {
	return names.length === 0 ? 'Personne assigné' : names.join(', ')
}
