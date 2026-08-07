import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'
const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

/**
 * Absences are read back through `GET /planning` (a `PlanningEntry` of kind
 * `absence` already carries every field the edit form needs — see the
 * planning design doc), so a mutation here only ever needs to invalidate the
 * planning window, never a dedicated absences list.
 */
function invalidatePlanning(queryClient: ReturnType<typeof useQueryClient>) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyMeta(query.queryKey)?._id === PLANNING_PATH,
	})
}

export function useCreateAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ABSENCES_PATH).mutationOptions,
		onSuccess: () => invalidatePlanning(queryClient),
	})
}

export function useUpdateAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ABSENCE_PATH).mutationOptions,
		onSuccess: () => invalidatePlanning(queryClient),
	})
}

export function useDeleteAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ABSENCE_PATH).mutationOptions,
		onSuccess: () => invalidatePlanning(queryClient),
	})
}

export type Absence = Schemas.AbsenceResponse
export type CreateAbsenceRequest = Schemas.CreateAbsenceRequest
export type UpdateAbsenceRequest = Schemas.UpdateAbsenceRequest
export type AbsenceKind = Schemas.AbsenceKind
