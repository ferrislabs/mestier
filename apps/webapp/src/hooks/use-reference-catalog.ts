import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const EMPLOYEE_PROFILES_PATH =
	'/api/v1/organizations/{organization_id}/employee-profiles'
const EQUIPMENT_PATH = '/api/v1/organizations/{organization_id}/equipment'
const SERVICE_RATES_PATH =
	'/api/v1/organizations/{organization_id}/service-rates'
const PRODUCTS_PATH = '/api/v1/organizations/{organization_id}/products'

type ReferenceListPath =
	| typeof MEMBERS_PATH
	| typeof EMPLOYEE_PROFILES_PATH
	| typeof EQUIPMENT_PATH
	| typeof SERVICE_RATES_PATH
	| typeof PRODUCTS_PATH

interface ReferenceCatalogOptions {
	members?: boolean
	employeeProfiles?: boolean
	equipment?: boolean
	serviceRates?: boolean
	products?: boolean
}

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function referenceListParams(organizationId: string) {
	return {
		path: { organization_id: organizationId },
		query: { page: 1, per_page: 100 },
	}
}

function invalidateReferenceList(
	queryClient: ReturnType<typeof useQueryClient>,
	path: ReferenceListPath,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyMeta(query.queryKey)?._id === path,
	})
}

function isReferenceEnabled(
	options: ReferenceCatalogOptions,
	key: keyof ReferenceCatalogOptions,
) {
	return options[key] ?? true
}

export function useReferenceCatalog(
	organizationId: string,
	options: ReferenceCatalogOptions = {},
) {
	const params = referenceListParams(organizationId)

	const members = useQuery({
		...window.tanstackApi.get(MEMBERS_PATH, params).queryOptions,
		enabled: isReferenceEnabled(options, 'members'),
	})
	const employeeProfiles = useQuery({
		...window.tanstackApi.get(EMPLOYEE_PROFILES_PATH, params).queryOptions,
		enabled: isReferenceEnabled(options, 'employeeProfiles'),
	})
	const equipment = useQuery({
		...window.tanstackApi.get(EQUIPMENT_PATH, params).queryOptions,
		enabled: isReferenceEnabled(options, 'equipment'),
	})
	const serviceRates = useQuery({
		...window.tanstackApi.get(SERVICE_RATES_PATH, params).queryOptions,
		enabled: isReferenceEnabled(options, 'serviceRates'),
	})
	const products = useQuery({
		...window.tanstackApi.get(PRODUCTS_PATH, params).queryOptions,
		enabled: isReferenceEnabled(options, 'products'),
	})

	return { members, employeeProfiles, equipment, serviceRates, products }
}

/** A single member by id — used by screens reached by direct link (e.g. the work-time page), which can't rely on the paginated list already holding it. */
export function useMember(memberId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get('/api/v1/members/{member_id}', {
			path: { member_id: memberId },
		}).queryOptions,
		enabled: enabled && Boolean(memberId),
	})
}

export function useCreateMember(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', MEMBERS_PATH).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, MEMBERS_PATH),
		onError: (error) => {
			console.error('[reference] failed to create member', error)
		},
		meta: { organizationId },
	})
}

export function useUpdateMember() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', '/api/v1/members/{member_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, MEMBERS_PATH),
	})
}

export function useDeleteMember() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', '/api/v1/members/{member_id}')
			.mutationOptions,
		onSuccess: () => {
			invalidateReferenceList(queryClient, MEMBERS_PATH)
			invalidateReferenceList(queryClient, EMPLOYEE_PROFILES_PATH)
		},
	})
}

/** Filling in a rate attaches the profile; the API replaces it wholesale on every call, so this doubles as "update". */
export function useUpsertEmployeeProfile() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation(
			'put',
			'/api/v1/members/{member_id}/employee-profile',
		).mutationOptions,
		onSuccess: () =>
			invalidateReferenceList(queryClient, EMPLOYEE_PROFILES_PATH),
	})
}

export function useRemoveEmployeeProfile() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation(
			'delete',
			'/api/v1/members/{member_id}/employee-profile',
		).mutationOptions,
		onSuccess: () =>
			invalidateReferenceList(queryClient, EMPLOYEE_PROFILES_PATH),
	})
}

export function useCreateEquipment(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', EQUIPMENT_PATH).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EQUIPMENT_PATH),
		meta: { organizationId },
	})
}

export function useUpdateEquipment() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', '/api/v1/equipment/{equipment_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EQUIPMENT_PATH),
	})
}

export function useDeleteEquipment() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', '/api/v1/equipment/{equipment_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EQUIPMENT_PATH),
	})
}

export function useCreateServiceRate(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', SERVICE_RATES_PATH).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, SERVICE_RATES_PATH),
		meta: { organizationId },
	})
}

export function useUpdateServiceRate() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation(
			'patch',
			'/api/v1/service-rates/{service_rate_id}',
		).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, SERVICE_RATES_PATH),
	})
}

export function useDeleteServiceRate() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation(
			'delete',
			'/api/v1/service-rates/{service_rate_id}',
		).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, SERVICE_RATES_PATH),
	})
}

export function useCreateProduct(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PRODUCTS_PATH).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, PRODUCTS_PATH),
		meta: { organizationId },
	})
}

export function useUpdateProduct() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', '/api/v1/products/{product_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, PRODUCTS_PATH),
	})
}

export function useDeleteProduct() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', '/api/v1/products/{product_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, PRODUCTS_PATH),
	})
}

export type Member = Schemas.MemberResponse
export type EmployeeProfile = Schemas.EmployeeResponse
export type Equipment = Schemas.EquipmentResponse
export type Product = Schemas.ProductResponse
export type ServiceRate = Schemas.ServiceRateResponse
export type ServiceRateUnit = Schemas.ServiceRateUnit
