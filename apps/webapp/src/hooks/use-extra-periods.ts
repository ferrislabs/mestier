import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import {
	fetchAllPages,
	LIST_PAGE_SIZE,
	readListPage,
} from '#/hooks/fetch-all-pages'
import { isForbiddenError } from '#/lib/api-error'

const EXTRAS_PATH = '/api/v1/organizations/{organization_id}/extra-periods'
const APPROVE_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve'
const REJECT_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject'
const CANCEL_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel'

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

function extrasPage(organizationId: string, page: number) {
	return window.tanstackApi.get(EXTRAS_PATH, {
		path: { organization_id: organizationId },
		query: { page, per_page: LIST_PAGE_SIZE },
	}).queryOptions
}

export function useExtraPeriods(organizationId: string, enabled = true) {
	return useQuery({
		queryKey: extrasPage(organizationId, 1).queryKey,
		queryFn: (context) =>
			fetchAllPages<Schemas.ExtraPeriodResponse>((page) =>
				readListPage<Schemas.ExtraPeriodResponse>(
					extrasPage(organizationId, page),
					context,
				),
			),
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: Error) =>
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
