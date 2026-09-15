import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidatePlanning } from '#/hooks/use-planning'
import type { Task } from '#/hooks/use-tasks'
import {
	applyBoardPatch,
	type UpdateTaskRequest,
} from '#/pages/planification/lib/board'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'

/**
 * The board is not a pager: it shows every root task at once, because a
 * column whose bottom half lives on page 2 is not a column. One generous
 * page instead of the list view's 20.
 */
export const BOARD_TASKS_PER_PAGE = 200

function boardTasksRequest(organizationId: string) {
	return {
		path: { organization_id: organizationId },
		query: { page: 1, per_page: BOARD_TASKS_PER_PAGE },
	}
}

/**
 * The board's own root-task read. Deliberately not `useRootTasks`: the
 * optimistic move below writes straight into this query's cache entry, so
 * the board owns the exact request — and therefore the exact key — rather
 * than sharing one with a screen that might change its page size.
 */
export function useBoardTasks(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(TASKS_PATH, boardTasksRequest(organizationId))
			.queryOptions,
		enabled: Boolean(organizationId),
	})
}

interface BoardTasksPage {
	data: Task[]
	pagination?: unknown
}

interface BoardPatchVariables {
	path: { organization_id: string; task_id: string }
	body: UpdateTaskRequest
}

/**
 * `PATCH /tasks/{id}` for every board gesture — a drop, an arrow key, a date
 * picked on a card — applied to the cached board the instant it is made and
 * rolled back whole if the server refuses.
 *
 * A separate hook from `usePatchTask` rather than an option on it: that one
 * is shared with the calendar and the task sheet, which have no board cache
 * to move cards around in, and giving it an optimistic update would move
 * cards for them too.
 *
 * The rollback restores the entire previous page, not just the card that
 * moved: a card's position is the order of the list around it, and putting
 * one element back where it was is only correct if nothing else shifted.
 */
export function useMoveBoardTask(organizationId: string) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(
		TASKS_PATH,
		boardTasksRequest(organizationId),
	).queryKey

	return useMutation({
		...window.tanstackApi.mutation('patch', TASK_PATH).mutationOptions,
		onMutate: async (variables) => {
			const { path, body } = variables as BoardPatchVariables
			await queryClient.cancelQueries({ queryKey })

			const previous = queryClient.getQueryData<BoardTasksPage>(queryKey)
			if (previous) {
				queryClient.setQueryData<BoardTasksPage>(queryKey, {
					...previous,
					data: applyBoardPatch(previous.data, path.task_id, body),
				})
			}

			return { previous }
		},
		onError: (_error, _variables, context) => {
			const previous = (context as { previous?: BoardTasksPage } | undefined)
				?.previous
			if (previous) queryClient.setQueryData(queryKey, previous)
		},
		onSettled: async () => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey }),
				invalidatePlanning(queryClient),
			])
		},
	})
}
