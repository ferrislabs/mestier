import {
	keepPreviousData,
	useMutation,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ADMIN_ACCESS_PATH = '/api/v1/admin/access'
const ADMIN_STATS_PATH = '/api/v1/admin/stats'
const ADMIN_ORGANIZATIONS_PATH = '/api/v1/admin/organizations'
const ADMIN_ORGANIZATION_PATH = '/api/v1/admin/organizations/{organization_id}'
const ADMIN_ORGANIZATION_FEATURE_PATH =
	'/api/v1/admin/organizations/{organization_id}/features/{flag}'
const ADMIN_USERS_PATH = '/api/v1/admin/users'
const ORGANIZATION_FEATURES_PATH =
	'/api/v1/organizations/{organization_id}/features'

const FEATURE_PATHS: readonly unknown[] = [
	ADMIN_STATS_PATH,
	ADMIN_ORGANIZATIONS_PATH,
	ADMIN_ORGANIZATION_PATH,
	ORGANIZATION_FEATURES_PATH,
]

export type AdminStats = Schemas.PlatformStatsResponse
export type AdminOrganizationDetail = Schemas.AdminOrganizationResponse
export type AdminOrganizationFeature = Schemas.AdminOrganizationFeatureResponse
export type AdminFeatureChange = Schemas.AdminFeatureChangeResponse

export interface AdminListParams {
	search: string
	page: number
	perPage: number
}

function listQuery(params: AdminListParams) {
	return {
		search: params.search === '' ? undefined : params.search,
		page: params.page,
		per_page: params.perPage,
	}
}

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

export function useAdminAccess() {
	return useQuery({
		...window.tanstackApi.get(ADMIN_ACCESS_PATH).queryOptions,
		staleTime: 5 * 60 * 1000,
		retry: false,
	})
}

export function useAdminStats() {
	return useQuery(window.tanstackApi.get(ADMIN_STATS_PATH).queryOptions)
}

export function useAdminOrganizations(params: AdminListParams) {
	return useQuery({
		...window.tanstackApi.get(ADMIN_ORGANIZATIONS_PATH, {
			query: listQuery(params),
		}).queryOptions,
		placeholderData: keepPreviousData,
	})
}

export function useAdminOrganization(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(ADMIN_ORGANIZATION_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useSetOrganizationFeature() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', ADMIN_ORGANIZATION_FEATURE_PATH)
			.mutationOptions,
		onSuccess: () =>
			queryClient.invalidateQueries({
				predicate: (query) => FEATURE_PATHS.includes(queryId(query.queryKey)),
			}),
	})
}

export function useAdminUsers(params: AdminListParams) {
	return useQuery({
		...window.tanstackApi.get(ADMIN_USERS_PATH, {
			query: listQuery(params),
		}).queryOptions,
		placeholderData: keepPreviousData,
	})
}
