import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const TASK_COMMENTS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/{task_id}/comments'
const TASK_COMMENT_PATH =
	'/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}'

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
		task_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isTaskCommentListQuery(
	queryKey: readonly unknown[],
	organizationId: string,
	taskId: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TASK_COMMENTS_PATH &&
		meta.path?.organization_id === organizationId &&
		meta.path?.task_id === taskId
	)
}

/**
 * One page of `taskId`'s comment thread — paginated, oldest first (see
 * `libs/handlers-planning/src/task_comment/list.rs`): the caller drives
 * `page` explicitly rather than this hook accumulating pages itself, so the
 * thread never silently ends up holding the entire history in memory (see
 * `apps/webapp/src/pages/planning/lib/comments.ts`'s `canLoadMoreComments`/
 * `canLoadOlderComments`, which this feeds).
 */
export function useTaskComments(
	organizationId: string,
	taskId: string,
	page: number,
	perPage = 20,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TASK_COMMENTS_PATH, {
			path: { organization_id: organizationId, task_id: taskId },
			query: { page, per_page: perPage },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId) && Boolean(taskId),
	})
}

function invalidateThread(
	queryClient: ReturnType<typeof useQueryClient>,
	organizationId: string,
	taskId: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isTaskCommentListQuery(query.queryKey, organizationId, taskId),
	})
}

export function useCreateTaskComment(organizationId: string, taskId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', TASK_COMMENTS_PATH).mutationOptions,
		onSuccess: () => invalidateThread(queryClient, organizationId, taskId),
		meta: { organizationId, taskId },
	})
}

/** `PATCH` — the backend answers `403` for anyone but the comment's own author (see `TaskCommentService::update_task_comment`); the UI only ever offers this on a comment whose `author_is_self` the server itself reports as `true`. */
export function useUpdateTaskComment(organizationId: string, taskId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', TASK_COMMENT_PATH).mutationOptions,
		onSuccess: () => invalidateThread(queryClient, organizationId, taskId),
	})
}

export function useDeleteTaskComment(organizationId: string, taskId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', TASK_COMMENT_PATH).mutationOptions,
		onSuccess: () => invalidateThread(queryClient, organizationId, taskId),
	})
}

export type TaskComment = Schemas.TaskCommentResponse
export type PaginationMetadata = Schemas.PaginationMetadata
