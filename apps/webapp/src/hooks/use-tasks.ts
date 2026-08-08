import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidatePlanning } from '#/hooks/use-planning'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'

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
