import {
	type QueryClient,
	useMutation,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const DEALS_PATH = '/api/v1/organizations/{organization_id}/deals'
const DEALS_SUMMARY_PATH =
	'/api/v1/organizations/{organization_id}/deals/summary'
const DEAL_PATH = '/api/v1/deals/{deal_id}'
const DEAL_STAGE_PATH = '/api/v1/deals/{deal_id}/stage'
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const DEAL_QUERY_PATHS: readonly unknown[] = [
	DEALS_PATH,
	DEALS_SUMMARY_PATH,
	DEAL_PATH,
]
const CUSTOMER_QUERY_PATHS: readonly unknown[] = [CUSTOMERS_PATH, CUSTOMER_PATH]

export type ApiDeal = Schemas.DealResponse
export type ApiDealStage = Schemas.DealStageName
export type ApiDealSummary = Schemas.DealSummaryResponse

interface DealListParams {
	page: number
	perPage: number
	customerId?: string
	stage?: ApiDealStage
	query?: string
}

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isQueryOf(
	paths: readonly unknown[],
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	if (!meta || !paths.includes(meta._id)) return false
	const queried = meta.path?.organization_id
	return !organizationId || queried === undefined || queried === organizationId
}

export function invalidateDealQueries(
	queryClient: QueryClient,
	organizationId?: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isQueryOf(DEAL_QUERY_PATHS, query.queryKey, organizationId),
	})
}

function invalidateCustomerQueries(
	queryClient: QueryClient,
	organizationId: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isQueryOf(CUSTOMER_QUERY_PATHS, query.queryKey, organizationId),
	})
}

function listParams(organizationId: string, params: DealListParams) {
	return {
		path: { organization_id: organizationId },
		query: {
			page: params.page,
			per_page: params.perPage,
			...(params.customerId ? { customer_id: params.customerId } : {}),
			...(params.stage ? { stage: params.stage } : {}),
			...(params.query ? { q: params.query } : {}),
		},
	}
}

export function useDealList(
	organizationId: string,
	params: DealListParams = { page: 1, perPage: 100 },
) {
	return useQuery({
		...window.tanstackApi.get(DEALS_PATH, listParams(organizationId, params))
			.queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useDealSummary(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(DEALS_SUMMARY_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCreateDeal(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', DEALS_PATH).mutationOptions,
		onSuccess: () => invalidateDealQueries(queryClient, organizationId),
		meta: { organizationId },
	})
}

export function useUpdateDeal(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', DEAL_PATH).mutationOptions,
		onSuccess: () => invalidateDealQueries(queryClient, organizationId),
	})
}

export function useMoveDeal(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', DEAL_STAGE_PATH).mutationOptions,
		onSuccess: () =>
			Promise.all([
				invalidateDealQueries(queryClient, organizationId),
				invalidateCustomerQueries(queryClient, organizationId),
			]),
	})
}

export function useDeleteDeal(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', DEAL_PATH).mutationOptions,
		onSuccess: () => invalidateDealQueries(queryClient, organizationId),
	})
}
