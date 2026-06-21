import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const CUSTOMER_CONTEXTS_PATH =
	'/api/v1/customers/{customer_id}/customer-contexts'
const CUSTOMER_CONTEXT_PATH = '/api/v1/customer-contexts/{customer_context_id}'
const CUSTOMER_CONTACTS_PATH = '/api/v1/customers/{customer_id}/contacts'
const CUSTOMER_CONTACT_PATH = '/api/v1/customer-contacts/{customer_contact_id}'

interface CustomerListParams {
	page: number
	perPage: number
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

function isCustomerListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === CUSTOMERS_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
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

function customerContactsKey(customerId: string) {
	return window.tanstackApi.get(CUSTOMER_CONTACTS_PATH, {
		path: { customer_id: customerId },
		query: { page: 1, per_page: 100 },
	}).queryKey
}

function customerContactKey(customerContactId: string) {
	return window.tanstackApi.get(CUSTOMER_CONTACT_PATH, {
		path: { customer_contact_id: customerContactId },
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

export function useCustomerContacts(customerId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(CUSTOMER_CONTACTS_PATH, {
			path: { customer_id: customerId },
			query: { page: 1, per_page: 100 },
		}).queryOptions,
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
					isCustomerListQuery(query.queryKey, organizationId),
			})
		},
		meta: { organizationId },
	})
}

export function useUpdateCustomer() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', CUSTOMER_PATH).mutationOptions,
		onSuccess: async (response) => {
			const customer = response.data
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isCustomerListQuery(query.queryKey, customer.organization_id),
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
						isCustomerListQuery(query.queryKey, organizationId),
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
		onSuccess: async (response) => {
			const customerContext = response.data
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
		onSuccess: async (response) => {
			const customerContext = response.data
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

export function useCreateCustomerContact() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CUSTOMER_CONTACTS_PATH)
			.mutationOptions,
		onSuccess: async (response) => {
			const customerContact = response.data
			await queryClient.invalidateQueries({
				queryKey: customerContactsKey(customerContact.customer_id),
			})
		},
	})
}

export function useUpdateCustomerContact() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', CUSTOMER_CONTACT_PATH)
			.mutationOptions,
		onSuccess: async (response) => {
			const customerContact = response.data
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customerContactsKey(customerContact.customer_id),
				}),
				queryClient.invalidateQueries({
					queryKey: customerContactKey(customerContact.id),
				}),
			])
		},
	})
}

export function useDeleteCustomerContact(customerId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', CUSTOMER_CONTACT_PATH)
			.mutationOptions,
		onSuccess: async (_res, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: customerContactsKey(customerId),
				}),
				queryClient.invalidateQueries({
					queryKey: customerContactKey(variables.path.customer_contact_id),
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
export type CustomerStatus = Schemas.CustomerStatus
export type CustomerContact = Schemas.CustomerContactResponse
export type CustomerContactPayload = Schemas.CreateCustomerContactRequest
export type CustomerContext = Schemas.CustomerContextResponse
export type CustomerContextPayload = Schemas.CreateCustomerContextRequest
export type FileUpload = Schemas.FileUploadResponse
export type PaginationMetadata = Schemas.PaginationMetadata
