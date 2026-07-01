import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const INVOICES_PATH = '/api/v1/organizations/{organization_id}/invoices'
const INVOICE_PATH = '/api/v1/invoices/{invoice_id}'
const INVOICE_STATUS_PATH = '/api/v1/invoices/{invoice_id}/status'

interface InvoiceListParams {
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

function isInvoiceListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === INVOICES_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

function listParams(organizationId: string, params: InvoiceListParams) {
	return {
		path: { organization_id: organizationId },
		query: { page: params.page, per_page: params.perPage },
	}
}

function invoiceKey(invoiceId: string) {
	return window.tanstackApi.get(INVOICE_PATH, {
		path: { invoice_id: invoiceId },
	}).queryKey
}

export function useInvoices(
	organizationId: string,
	params: InvoiceListParams = { page: 1, perPage: 100 },
) {
	return useQuery(
		window.tanstackApi.get(INVOICES_PATH, listParams(organizationId, params))
			.queryOptions,
	)
}

export function useInvoice(invoiceId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(INVOICE_PATH, {
			path: { invoice_id: invoiceId },
		}).queryOptions,
		enabled: enabled && Boolean(invoiceId),
	})
}

export function useCreateInvoice(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICES_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, organizationId),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
		meta: { organizationId },
	})
}

export function useUpdateInvoice(invoiceId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', INVOICE_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoiceId),
				}),
			])
		},
	})
}

export function useUpdateInvoiceStatus(invoiceId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', INVOICE_STATUS_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoiceId),
				}),
			])
		},
	})
}

export function useDeleteInvoice(organizationId?: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', INVOICE_PATH).mutationOptions,
		onSuccess: async (_response, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, organizationId),
				}),
				queryClient.removeQueries({
					queryKey: invoiceKey(variables.path.invoice_id),
				}),
			])
		},
	})
}

const CONVERT_QUOTE_PATH =
	'/api/v1/quotes/{quote_id}/convert-to-invoice' as const
const DEPOSIT_INVOICE_PATH =
	'/api/v1/invoices/{invoice_id}/deposit-invoice' as const
const BALANCE_INVOICE_PATH =
	'/api/v1/invoices/{invoice_id}/balance-invoice' as const
const ISSUE_INVOICE_PATH = '/api/v1/invoices/{invoice_id}/issue' as const
const SHARE_INVOICE_PATH = '/api/v1/invoices/{invoice_id}/share' as const

export function useConvertQuoteToInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', CONVERT_QUOTE_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
	})
}

export function useCreateDepositInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', DEPOSIT_INVOICE_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
	})
}

export function useCreateBalanceInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', BALANCE_INVOICE_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
	})
}

export function useIssueInvoice(invoiceId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ISSUE_INVOICE_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isInvoiceListQuery(query.queryKey, invoice.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoiceId),
				}),
			])
		},
	})
}

export function useShareInvoice() {
	return useMutation({
		...window.tanstackApi.mutation('post', SHARE_INVOICE_PATH).mutationOptions,
	})
}

export type Invoice = Schemas.InvoiceResponse
export type InvoicePayload = Schemas.CreateInvoiceRequest
export type InvoiceLinePayload = Schemas.InvoiceLineRequest
export type InvoiceStatus = Schemas.InvoiceStatus
export type PaginationMetadata = Schemas.PaginationMetadata
