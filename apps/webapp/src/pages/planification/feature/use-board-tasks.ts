import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidatePlanning } from '#/hooks/use-planning'
import type { Task } from '#/hooks/use-tasks'
import {
	applyBoardPatch,
	type UpdateTaskRequest,
} from '#/pages/planification/lib/board'
import {
	type BoardFilters,
	boardFiltersToQuery,
} from '#/pages/planification/lib/board-filters'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'

/**
 * The board is not a pager: it shows every root task at once, because a
 * column whose bottom half lives on page 2 is not a column. One generous
 * page instead of the list view's 20.
 */
export const BOARD_TASKS_PER_PAGE = 200

/**
 * The one request both hooks below build, filters included.
 *
 * The filters belong in here rather than being applied to the response,
 * and not only because a client-side filter would lie about the column
 * counts: this object *is* the query key, and the optimistic move writes
 * into the entry that key names. A filter that did not travel this far
 * would leave the move patching the unfiltered board while the screen shows
 * the filtered one.
 */
function boardTasksRequest(organizationId: string, filters: BoardFilters) {
	return {
		path: { organization_id: organizationId },
		query: boardFiltersToQuery(filters, BOARD_TASKS_PER_PAGE),
	}
}

/**
 * The board's own root-task read. Deliberately not `useRootTasks`: the
 * optimistic move below writes straight into this query's cache entry, so
 * the board owns the exact request — and therefore the exact key — rather
 * than sharing one with a screen that might change its page size.
 */
export function useBoardTasks(organizationId: string, filters: BoardFilters) {
	return useQuery({
		...window.tanstackApi.get(
			TASKS_PATH,
			boardTasksRequest(organizationId, filters),
		).queryOptions,
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
 *
 * Takes the same `filters` as {@link useBoardTasks} because it has to name
 * the same cache entry: dropping a card on a board narrowed to one project
 * must patch that project's page, not the unfiltered one sitting beside it
 * in the cache.
 */
export function useMoveBoardTask(
	organizationId: string,
	filters: BoardFilters,
) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(
		TASKS_PATH,
		boardTasksRequest(organizationId, filters),
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
