import type { QueryClient } from '@tanstack/react-query'

const MAINTENANCE_PREFIX =
	'/api/v1/organizations/{organization_id}/maintenance/'
const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'

function queryId(queryKey: readonly unknown[]): string | null {
	const meta = queryKey[0]
	if (typeof meta !== 'object' || meta === null) return null
	const id = (meta as { _id?: unknown })._id
	return typeof id === 'string' ? id : null
}

/**
 * Everything the maintenance screens read: the recap, the grid, the visit
 * queue. A visit validated or a line planned moves the due date of lines on
 * all of them, so they refresh together rather than one by one.
 */
export function invalidateMaintenance(queryClient: QueryClient) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			queryId(query.queryKey)?.startsWith(MAINTENANCE_PREFIX) ?? false,
	})
}

/** The board and the task lists, which gain a task when a line is planned. */
export function invalidateTasks(queryClient: QueryClient) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			queryId(query.queryKey)?.startsWith(TASKS_PATH) ?? false,
	})
}
