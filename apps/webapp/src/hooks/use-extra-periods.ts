import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const EXTRAS_PATH = '/api/v1/organizations/{organization_id}/extra-periods'
const APPROVE_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve'
const REJECT_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject'
const CANCEL_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel'

const EXTRAS_LIST_PER_PAGE = 100

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateExtraReads(queryClient: ReturnType<typeof useQueryClient>) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === EXTRAS_PATH,
	})
}

export function useExtraPeriods(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(EXTRAS_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: EXTRAS_LIST_PER_PAGE },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useCreateExtraPeriod() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', EXTRAS_PATH).mutationOptions,
		onSuccess: () => invalidateExtraReads(queryClient),
	})
}

export function useApproveExtraPeriod() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', APPROVE_PATH).mutationOptions,
		onSuccess: () => invalidateExtraReads(queryClient),
	})
}

export function useRejectExtraPeriod() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', REJECT_PATH).mutationOptions,
		onSuccess: () => invalidateExtraReads(queryClient),
	})
}

export function useCancelExtraPeriod() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CANCEL_PATH).mutationOptions,
		onSuccess: () => invalidateExtraReads(queryClient),
	})
}

export type ExtraPeriodRecord = Schemas.ExtraPeriodResponse
export type CreateExtraPeriodRequest = Schemas.CreateExtraPeriodRequest
