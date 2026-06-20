import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const CUSTOMER_CONTEXTS_PATH =
	'/api/v1/customers/{customer_id}/customer-contexts'
const CUSTOMER_CONTEXT_PATH = '/api/v1/customer-contexts/{customer_context_id}'

interface CustomerListParams {
	page: number
	perPage: number
}

function listParams(organizationId: string, params: CustomerListParams) {
	return {
		path: { organization_id: organizationId },
		query: { page: params.page, per_page: params.perPage },
	}
}

function customerContextsParams(customerId: string) {
	return {
		path: { customer_id: customerId },
		query: { page: 1, per_page: 100 },
	}
}

function customerKey(customerId: string) {
	return window.tanstackApi.get(CUSTOMER_PATH, {
		path: { customer_id: customerId },
	}).queryKey
}

function customerContextsKey(customerId: string) {
	return window.tanstackApi.get(
		CUSTOMER_CONTEXTS_PATH,
		customerContextsParams(customerId),
	).queryKey
}

function customerContextKey(customerContextId: string) {
	return window.tanstackApi.get(CUSTOMER_CONTEXT_PATH, {
		path: { customer_context_id: customerContextId },
	}).queryKey
}

export function useCustomers(
	organizationId: string,
	params: CustomerListParams = { page: 1, perPage: 100 },
) {
	return useQuery(
		window.tanstackApi.get(CUSTOMERS_PATH, listParams(organizationId, params))
			.queryOptions,
	)
}

export function useCustomer(customerId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(CUSTOMER_PATH, {
			path: { customer_id: customerId },
		}).queryOptions,
		enabled: enabled && Boolean(customerId),
	})
}

export function useCustomerContexts(customerId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(
			CUSTOMER_CONTEXTS_PATH,
			customerContextsParams(customerId),
		).queryOptions,
		enabled: enabled && Boolean(customerId),
	})
}

export function useCreateCustomer(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CUSTOMERS_PATH).mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({
				predicate: (query) =>
					query.queryKey[0]?._id === CUSTOMERS_PATH &&
					query.queryKey[0]?.path?.organization_id === organizationId,
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
					predicate: (query) =>
						query.queryKey[0]?._id === CUSTOMERS_PATH &&
						query.queryKey[0]?.path?.organization_id ===
							customer.organization_id,
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
					predicate: (query) =>
						query.queryKey[0]?._id === CUSTOMERS_PATH &&
						query.queryKey[0]?.path?.organization_id === organizationId,
				}),
				queryClient.invalidateQueries({
					queryKey: customerKey(variables.path.customer_id),
				}),
			])
		},
	})
}

export function useCreateCustomerContext() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CUSTOMER_CONTEXTS_PATH)
			.mutationOptions,
		onSuccess: async (customerContext) => {
			await queryClient.invalidateQueries({
				queryKey: customerContextsKey(customerContext.customer_id),
			})
		},
	})
}

export function useUpdateCustomerContext() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', CUSTOMER_CONTEXT_PATH)
			.mutationOptions,
		onSuccess: async (customerContext) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customerContextsKey(customerContext.customer_id),
				}),
				queryClient.invalidateQueries({
					queryKey: customerContextKey(customerContext.id),
				}),
			])
		},
	})
}

export function useDeleteCustomerContext(customerId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', CUSTOMER_CONTEXT_PATH)
			.mutationOptions,
		onSuccess: async (_res, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customerContextsKey(customerId),
				}),
				queryClient.invalidateQueries({
					queryKey: customerContextKey(variables.path.customer_context_id),
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
export type CustomerContext = Schemas.CustomerContextResponse
export type CustomerContextPayload = Schemas.CreateCustomerContextRequest
export type FileUpload = Schemas.FileUploadResponse
export type PaginationMetadata = Schemas.PaginationMetadata
