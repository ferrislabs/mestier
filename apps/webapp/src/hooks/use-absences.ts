import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import {
	fetchAllPages,
	LIST_PAGE_SIZE,
	readListPage,
} from '#/hooks/fetch-all-pages'

const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'
const APPROVE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}/approve'
const REJECT_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}/reject'
const CANCEL_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}/cancel'
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

function absencesPage(organizationId: string, page: number) {
	return window.tanstackApi.get(ABSENCES_PATH, {
		path: { organization_id: organizationId },
		query: { page, per_page: LIST_PAGE_SIZE },
	}).queryOptions
}

/**
 * Every absence the caller may read, all pages fetched: the whole
 * organization for a planning or reference manager, only their own
 * otherwise (the API filters).
 */
export function useAbsences(organizationId: string, enabled = true) {
	return useQuery({
		queryKey: absencesPage(organizationId, 1).queryKey,
		queryFn: (context) =>
			fetchAllPages<Schemas.AbsenceResponse>((page) =>
				readListPage<Schemas.AbsenceResponse>(
					absencesPage(organizationId, page),
					context,
				),
			),
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

export function useApproveAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', APPROVE_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export function useRejectAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', REJECT_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export function useCancelAbsence() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CANCEL_PATH).mutationOptions,
		onSuccess: () => invalidateAbsenceReads(queryClient),
	})
}

export type Absence = Schemas.AbsenceResponse
export type CreateAbsenceRequest = Schemas.CreateAbsenceRequest
export type UpdateAbsenceRequest = Schemas.UpdateAbsenceRequest
export type AbsenceKind = Schemas.AbsenceKind
