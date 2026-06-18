import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const PROPERTIES_PATH = '/api/v1/customers/{customer_id}/properties'
const PROPERTY_PATH = '/api/v1/properties/{property_id}'

function listParams(organizationId: string) {
	return {
		path: { organization_id: organizationId },
		query: { page: 1, per_page: 100 },
	}
}

function propertiesParams(customerId: string) {
	return {
		path: { customer_id: customerId },
		query: { page: 1, per_page: 100 },
	}
}

function customersKey(organizationId: string) {
	return window.tanstackApi.get(CUSTOMERS_PATH, listParams(organizationId))
		.queryKey
}

function customerKey(customerId: string) {
	return window.tanstackApi.get(CUSTOMER_PATH, {
		path: { customer_id: customerId },
	}).queryKey
}

function propertiesKey(customerId: string) {
	return window.tanstackApi.get(PROPERTIES_PATH, propertiesParams(customerId))
		.queryKey
}

function propertyKey(propertyId: string) {
	return window.tanstackApi.get(PROPERTY_PATH, {
		path: { property_id: propertyId },
	}).queryKey
}

export function useCustomers(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(CUSTOMERS_PATH, listParams(organizationId))
			.queryOptions,
	)
}

export function useCustomer(customerId: string) {
	return useQuery(
		window.tanstackApi.get(CUSTOMER_PATH, {
			path: { customer_id: customerId },
		}).queryOptions,
	)
}

export function useCustomerProperties(customerId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(PROPERTIES_PATH, propertiesParams(customerId))
			.queryOptions,
		enabled: enabled && Boolean(customerId),
	})
}

export function useCreateCustomer(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CUSTOMERS_PATH).mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({
				queryKey: customersKey(organizationId),
			})
		},
		meta: { organizationId },
	})
}

export function useUpdateCustomer() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', CUSTOMER_PATH).mutationOptions,
		onSuccess: async (customer) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customersKey(customer.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: customerKey(customer.id),
				}),
			])
		},
	})
}

export function useDeleteCustomer(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', CUSTOMER_PATH).mutationOptions,
		onSuccess: async (_res, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customersKey(organizationId),
				}),
				queryClient.invalidateQueries({
					queryKey: customerKey(variables.path.customer_id),
				}),
			])
		},
	})
}

export function useCreateProperty() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PROPERTIES_PATH).mutationOptions,
		onSuccess: async (property) => {
			await queryClient.invalidateQueries({
				queryKey: propertiesKey(property.customer_id),
			})
		},
	})
}

export function useUpdateProperty() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', PROPERTY_PATH).mutationOptions,
		onSuccess: async (property) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: propertiesKey(property.customer_id),
				}),
				queryClient.invalidateQueries({
					queryKey: propertyKey(property.id),
				}),
			])
		},
	})
}

export function useDeleteProperty(customerId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', PROPERTY_PATH).mutationOptions,
		onSuccess: async (_res, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: propertiesKey(customerId),
				}),
				queryClient.invalidateQueries({
					queryKey: propertyKey(variables.path.property_id),
				}),
			])
		},
	})
}

export function useUploadFile() {
	return useMutation({
		mutationFn: async (file: File) => {
			const body = await file.arrayBuffer()
			return window.api.post('/api/v1/files', {
				body,
				header: {
					'Content-Type': file.type || 'application/octet-stream',
				},
			} as never)
		},
	})
}

export type Customer = Schemas.CustomerResponse
export type CustomerPayload = Schemas.CreateCustomerRequest
export type Property = Schemas.PropertyResponse
export type PropertyPayload = Schemas.CreatePropertyRequest
export type FileUpload = Schemas.FileUploadResponse
