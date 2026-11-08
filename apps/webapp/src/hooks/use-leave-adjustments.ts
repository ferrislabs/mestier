import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const ADJUSTMENTS_PATH =
	'/api/v1/organizations/{organization_id}/leave-adjustments'
const ADJUSTMENT_PATH =
	'/api/v1/organizations/{organization_id}/leave-adjustments/{adjustment_id}'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateAdjustmentReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === ADJUSTMENTS_PATH,
	})
}

export function useLeaveAdjustments(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(ADJUSTMENTS_PATH, {
			path: { organization_id: organizationId },
			query: {},
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useCreateLeaveAdjustment() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ADJUSTMENTS_PATH).mutationOptions,
		onSuccess: () => invalidateAdjustmentReads(queryClient),
	})
}

export function useDeleteLeaveAdjustment() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ADJUSTMENT_PATH).mutationOptions,
		onSuccess: () => invalidateAdjustmentReads(queryClient),
	})
}

export type LeaveAdjustmentRecord = Schemas.LeaveAdjustmentResponse
export type CreateLeaveAdjustmentRequest = Schemas.CreateLeaveAdjustmentRequest
