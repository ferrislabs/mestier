import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const EMPLOYEES_PATH = '/api/v1/organizations/{organization_id}/employees'
const EQUIPMENT_PATH = '/api/v1/organizations/{organization_id}/equipment'
const SERVICE_RATES_PATH =
	'/api/v1/organizations/{organization_id}/service-rates'

type ReferenceListPath =
	| typeof EMPLOYEES_PATH
	| typeof EQUIPMENT_PATH
	| typeof SERVICE_RATES_PATH

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
		predicate: (query) => query.queryKey[0]?._id === path,
	})
}

export function useReferenceCatalog(organizationId: string) {
	const params = referenceListParams(organizationId)

	const employees = useQuery(
		window.tanstackApi.get(EMPLOYEES_PATH, params).queryOptions,
	)
	const equipment = useQuery(
		window.tanstackApi.get(EQUIPMENT_PATH, params).queryOptions,
	)
	const serviceRates = useQuery(
		window.tanstackApi.get(SERVICE_RATES_PATH, params).queryOptions,
	)

	return { employees, equipment, serviceRates }
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

export type Employee = Schemas.EmployeeResponse
export type Equipment = Schemas.EquipmentResponse
export type ServiceRate = Schemas.ServiceRateResponse
export type ServiceRateUnit = Schemas.ServiceRateUnit
