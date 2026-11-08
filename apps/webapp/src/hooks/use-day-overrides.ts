import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const OVERRIDES_PATH = '/api/v1/organizations/{organization_id}/day-overrides'
const OVERRIDE_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}'
const FEED_PATH = '/api/v1/organizations/{organization_id}/hr/planning-feed'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateOverrideReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const id = queryKeyId(query.queryKey)
			return id === OVERRIDES_PATH || id === FEED_PATH
		},
	})
}

export interface OverridePeriodQuery {
	from: string
	to: string
	memberId?: string
}

export function useDayOverrides(
	organizationId: string,
	period: OverridePeriodQuery,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(OVERRIDES_PATH, {
			path: { organization_id: organizationId },
			query: {
				from: period.from,
				to: period.to,
				...(period.memberId ? { member_id: period.memberId } : {}),
			},
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function usePutDayOverride() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', OVERRIDE_PATH).mutationOptions,
		onSuccess: () => invalidateOverrideReads(queryClient),
	})
}

export function useDeleteDayOverride() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', OVERRIDE_PATH).mutationOptions,
		onSuccess: () => invalidateOverrideReads(queryClient),
	})
}

export type DayOverrideResponse = Schemas.DayOverrideResponse
export type PutDayOverrideRequest = Schemas.PutDayOverrideRequest
