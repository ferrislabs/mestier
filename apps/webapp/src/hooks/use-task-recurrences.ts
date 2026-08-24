import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidatePlanning } from '#/hooks/use-planning'

const TASK_RECURRENCES_PATH =
	'/api/v1/organizations/{organization_id}/task-recurrences'
const TASK_RECURRENCE_PATH = '/api/v1/task-recurrences/{task_recurrence_id}'

/**
 * `POST /task-recurrences` — creates the series and materializes its
 * occurrences up to the horizon in one call (see the backend's
 * `TaskRecurrenceService::create_recurrence`). Invalidates the planning
 * views the same way {@link import('#/hooks/use-tasks').useCreateTask} does:
 * a series shows up as a batch of ordinary tasks, not through a channel of
 * its own.
 */
export function useCreateTaskRecurrence(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', TASK_RECURRENCES_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidatePlanning(queryClient)
		},
		meta: { organizationId },
	})
}

/**
 * `DELETE /task-recurrences/{id}` — removes the series and its future
 * occurrences, in one transaction; past occurrences are left standing (see
 * the backend's `TaskRecurrenceService::delete_recurrence`).
 */
export function useDeleteTaskRecurrence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', TASK_RECURRENCE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidatePlanning(queryClient)
		},
	})
}

export type TaskRecurrence = Schemas.TaskRecurrenceResponse
export type CreateTaskRecurrenceRequest = Schemas.CreateTaskRecurrenceRequest
