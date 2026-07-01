import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const EMPLOYEES_PATH = '/api/v1/organizations/{organization_id}/employees'
const EQUIPMENT_PATH = '/api/v1/organizations/{organization_id}/equipment'
const SERVICE_RATES_PATH =
	'/api/v1/organizations/{organization_id}/service-rates'
const PRODUCTS_PATH = '/api/v1/organizations/{organization_id}/products'

type ReferenceListPath =
	| typeof EMPLOYEES_PATH
	| typeof EQUIPMENT_PATH
	| typeof SERVICE_RATES_PATH
	| typeof PRODUCTS_PATH

interface ReferenceCatalogOptions {
	employees?: boolean
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
	const hasOrg = Boolean(organizationId)

	const employees = useQuery({
		...window.tanstackApi.get(EMPLOYEES_PATH, params).queryOptions,
		enabled: hasOrg && isReferenceEnabled(options, 'employees'),
	})
	const equipment = useQuery({
		...window.tanstackApi.get(EQUIPMENT_PATH, params).queryOptions,
		enabled: hasOrg && isReferenceEnabled(options, 'equipment'),
	})
	const serviceRates = useQuery({
		...window.tanstackApi.get(SERVICE_RATES_PATH, params).queryOptions,
		enabled: hasOrg && isReferenceEnabled(options, 'serviceRates'),
	})
	const products = useQuery({
		...window.tanstackApi.get(PRODUCTS_PATH, params).queryOptions,
		enabled: hasOrg && isReferenceEnabled(options, 'products'),
	})

	return { employees, equipment, serviceRates, products }
}

export function useCreateEmployee(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', EMPLOYEES_PATH).mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EMPLOYEES_PATH),
		onError: (error) => {
			console.error('[reference] failed to create employee', error)
		},
		meta: { organizationId },
	})
}

export function useUpdateEmployee() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', '/api/v1/employees/{employee_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EMPLOYEES_PATH),
	})
}

export function useDeleteEmployee() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', '/api/v1/employees/{employee_id}')
			.mutationOptions,
		onSuccess: () => invalidateReferenceList(queryClient, EMPLOYEES_PATH),
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

export type Employee = Schemas.EmployeeResponse
export type Equipment = Schemas.EquipmentResponse
export type Product = Schemas.ProductResponse
export type ServiceRate = Schemas.ServiceRateResponse
export type ServiceRateUnit = Schemas.ServiceRateUnit
