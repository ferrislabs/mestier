import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'
const AVAILABILITY_PATH =
	'/api/v1/organizations/{organization_id}/planning/availability'

/** An inclusive `[from, to]` window of ISO calendar days (`YYYY-MM-DD`), capped at 92 days by the API. */
export interface PlanningRange {
	from: string
	to: string
}

/**
 * Reads the team planning — resources, entries and work time — for `range`.
 * `from`/`to` are required by the API; see `computeWindow` for how a
 * `view`/`date` pair in the URL turns into this range.
 */
export function usePlanning(
	organizationId: string,
	range: PlanningRange,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(PLANNING_PATH, {
			path: { organization_id: organizationId },
			query: { from: range.from, to: range.to },
		}).queryOptions,
		enabled:
			enabled &&
			Boolean(organizationId) &&
			Boolean(range.from) &&
			Boolean(range.to),
	})
}

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isPlanningQuery(queryKey: readonly unknown[]) {
	return queryKeyMeta(queryKey)?._id === PLANNING_PATH
}

/**
 * Invalidates the planning grid's query — exported so mutations that
 * change a task from *outside* the grid (the task form, subtasks, `PATCH`
 * from the detail sheet) refresh it too, the same way a drag & drop already
 * does via {@link useMoveTask}.
 */
export function invalidatePlanning(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => isPlanningQuery(query.queryKey),
	})
}

/**
 * Checks every resource's availability over a candidate window — called
 * imperatively from a drag & drop or an absence-create gesture, never as a
 * `useQuery`: the caller decides in response to a user action, not on
 * render (see the planning design doc's "Avertissements" section). A `GET`
 * driven through `mutation()` so it can be awaited on demand with
 * `mutateAsync`.
 */
export function useCheckAvailability() {
	return useMutation({
		...window.tanstackApi.mutation('get', AVAILABILITY_PATH).mutationOptions,
	})
}

/**
 * The transactional `PATCH` a drag & drop gesture (or an assignee removal
 * going through the same path) sends — see the planning design doc's "Drag &
 * drop" decision: one endpoint reprograms and reassigns at once, so there is
 * no partial write to roll back on cancel.
 */
export function useMoveTask() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', TASK_PATH).mutationOptions,
		onSuccess: () => invalidatePlanning(queryClient),
	})
}

export type PlanningResource = Schemas.PlanningResourceResponse
export type PlanningEntry = Schemas.PlanningEntryResponse
export type PlanningWorkTime = Schemas.PlanningWorkTimeResponse
export type PlanningResponse = Schemas.PlanningResponse
export type AvailabilityResponse = Schemas.AvailabilityResponse
export type AvailabilityResource = Schemas.AvailabilityResourceResponse
export type ConflictResponse = Schemas.ConflictResponse
export type PatchTaskResponse = Schemas.PatchTaskResponse
export type AssigneeRef = Schemas.AssigneeRefRequest
export type CreatedEmployee = Schemas.EmployeeResponse
