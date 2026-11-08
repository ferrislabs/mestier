import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const SURCHARGES_PATH = '/api/v1/organizations/{organization_id}/surcharges'
const SURCHARGE_PATH =
	'/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}'
const ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/archive'
const RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/restore'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateSurchargeReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === SURCHARGES_PATH,
	})
}

export function useSurcharges(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(SURCHARGES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useCreateSurcharge() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', SURCHARGES_PATH).mutationOptions,
		onSuccess: () => invalidateSurchargeReads(queryClient),
	})
}

export function useUpdateSurcharge() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', SURCHARGE_PATH).mutationOptions,
		onSuccess: () => invalidateSurchargeReads(queryClient),
	})
}

export function useArchiveSurcharge() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidateSurchargeReads(queryClient),
	})
}

export function useRestoreSurcharge() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', RESTORE_PATH).mutationOptions,
		onSuccess: () => invalidateSurchargeReads(queryClient),
	})
}

export type SurchargeRecord = Schemas.SurchargeResponse
export type SurchargeRequest = Schemas.SurchargeRequest
