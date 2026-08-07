import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'
const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'

/**
 * The API paginates (`MAX_PER_PAGE = 100` — see `libs/pagination`, an
 * endpoint this workstream doesn't own and the design doc keeps
 * unchanged), and doesn't filter by employee. The HR screen fetches this
 * one page and filters by `employee_id` client-side (see
 * `EmployeeWorkTimeFeature`) — fine at artisan/SME scale, but an
 * organization with more than 100 absences on the books would silently
 * lose the tail. Flagged rather than worked around: fixing it for real
 * means either an `employee_id` filter or real pagination on the list
 * endpoint, both backend changes out of this workstream's scope.
 */
const ABSENCES_LIST_PER_PAGE = 100

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
 * Absences are also read back through `GET /planning` (a `PlanningEntry` of
 * kind `absence` carries every field the planning grid needs — see the
 * planning design doc), so a mutation here invalidates both that window and
 * this module's own list.
 */
function invalidateAbsenceReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const id = queryKeyMeta(query.queryKey)?._id
			return id === PLANNING_PATH || id === ABSENCES_PATH
		},
	})
}

/** Every absence in the organization — see {@link ABSENCES_LIST_PER_PAGE}'s doc for the pagination caveat. */
export function useAbsences(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(ABSENCES_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: ABSENCES_LIST_PER_PAGE },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

export function useCreateAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ABSENCES_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export function useUpdateAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ABSENCE_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export function useDeleteAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ABSENCE_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export type Absence = Schemas.AbsenceResponse
export type CreateAbsenceRequest = Schemas.CreateAbsenceRequest
export type UpdateAbsenceRequest = Schemas.UpdateAbsenceRequest
export type AbsenceKind = Schemas.AbsenceKind
