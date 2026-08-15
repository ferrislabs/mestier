import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidatePlanning } from '#/hooks/use-planning'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'
const BULK_ASSIGN_TASKS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/bulk-assign'

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
		parent_task_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isTaskListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TASKS_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

/**
 * A root task's subtasks — `GET /tasks?parent_task_id=` (see the planning
 * remodel design doc's API section). `enabled` defaults to `Boolean(taskId)`
 * so a root task not yet loaded, or a subtask (which never has its own
 * children — see {@link import('#/pages/planning/lib/subtasks').canAddSubtask}),
 * simply doesn't fetch rather than the caller having to remember to gate it.
 *
 * Always requests page 1 at `per_page: 100` — this is a single, unpaginated
 * fetch, not the first page of a pager the caller can advance. A parent
 * with more than 100 subtasks would silently lose the rest from the sheet's
 * "Sous-tâches" tab: nothing surfaces that truncation today. Left as-is
 * rather than paginated up front — 100 direct children of one task is not a
 * shape this module has seen in practice — but a real cap, not a
 * theoretical one, so it belongs in a comment rather than assumed away.
 */
export function useSubtasks(
	organizationId: string,
	parentTaskId: string,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TASKS_PATH, {
			path: { organization_id: organizationId },
			query: { parent_task_id: parentTaskId, page: 1, per_page: 100 },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId) && Boolean(parentTaskId),
	})
}

/**
 * The task list view's root rows — `GET /tasks` with no `parent_task_id`
 * lists root tasks and populates each one's `child_count`, computed
 * server-side without loading the hierarchy (see the planning remodel
 * design doc's API section). Paginated, unlike {@link useSubtasks}: the
 * list view's whole reason to exist is browsing every task, so it can't
 * assume a small fixed page the way a single task's subtasks tab can.
 */
export function useRootTasks(
	organizationId: string,
	page: number,
	perPage: number,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TASKS_PATH, {
			path: { organization_id: organizationId },
			query: { page, per_page: perPage },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

export function useTask(
	organizationId: string,
	taskId: string,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TASK_PATH, {
			path: { organization_id: organizationId, task_id: taskId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId) && Boolean(taskId),
	})
}

/**
 * `POST /tasks` — a task is always created bare (no `assignees`/`label_ids`
 * field on `CreateTaskRequest`, see `apps/webapp/src/pages/planning/lib/
 * task-form.ts`'s own doc), so the caller follows up with
 * {@link usePatchTask} when the create form picked assignees or labels.
 */
export function useCreateTask(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', TASKS_PATH).mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) => isTaskListQuery(query.queryKey, organizationId),
				}),
				invalidatePlanning(queryClient),
			])
		},
		meta: { organizationId },
	})
}

/**
 * `PATCH /tasks/{id}` — the general-purpose task edit, distinct from
 * `useMoveTask` in `use-planning.ts` only by call site (the task form and
 * the immediate follow-up patch after a create, rather than a grid drag &
 * drop); both hit the same endpoint and both invalidate the grid.
 */
export function usePatchTask() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', TASK_PATH).mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) => isTaskListQuery(query.queryKey),
				}),
				invalidatePlanning(queryClient),
			])
		},
	})
}

/**
 * `POST /tasks/bulk-assign` — replaces every named task's complete
 * assignee set in one transaction, all or nothing (see the backend's
 * `TaskService::bulk_assign_tasks`'s own doc). Same invalidation shape as
 * {@link usePatchTask}: the task list and the grid both show assignees.
 */
export function useBulkAssignTasks() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', BULK_ASSIGN_TASKS_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) => isTaskListQuery(query.queryKey),
				}),
				invalidatePlanning(queryClient),
			])
		},
	})
}

export function useDeleteTask() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', TASK_PATH).mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) => isTaskListQuery(query.queryKey),
				}),
				invalidatePlanning(queryClient),
			])
		},
	})
}

export type Task = Schemas.TaskResponse
export type CreateTaskRequest = Schemas.CreateTaskRequest
export type UpdateTaskRequest = Schemas.UpdateTaskRequest
export type PaginationMetadata = Schemas.PaginationMetadata
