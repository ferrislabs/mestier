import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const INVOICES_PATH = '/api/v1/organizations/{organization_id}/invoices'
const OUTSTANDING_PATH =
	'/api/v1/organizations/{organization_id}/invoices/outstanding'
const INVOICE_PATH = '/api/v1/invoices/{invoice_id}'
const INVOICE_BALANCE_PATH = '/api/v1/invoices/{invoice_id}/balance'
const INVOICE_CREDIT_NOTES_PATH = '/api/v1/invoices/{invoice_id}/credit-notes'
const INVOICE_PAYMENTS_PATH = '/api/v1/invoices/{invoice_id}/payments'
const INVOICE_PAYMENT_PATH = '/api/v1/invoice-payments/{invoice_payment_id}'
const INVOICE_ISSUE_PATH = '/api/v1/invoices/{invoice_id}/issue'
const INVOICE_CANCEL_PATH = '/api/v1/invoices/{invoice_id}/cancel'
const PROJECT_INVOICE_DEPOSIT_PATH =
	'/api/v1/projects/{project_id}/invoices/deposit'
const PROJECT_INVOICE_FINAL_PATH =
	'/api/v1/projects/{project_id}/invoices/final'

interface InvoiceListParams {
	page: number
	perPage: number
}

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
		invoice_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isOrganizationQuery(
	pathId: string,
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === pathId &&
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

function invoiceBalanceKey(invoiceId: string) {
	return window.tanstackApi.get(INVOICE_BALANCE_PATH, {
		path: { invoice_id: invoiceId },
	}).queryKey
}

function invoiceCreditNotesKey(invoiceId: string) {
	return window.tanstackApi.get(INVOICE_CREDIT_NOTES_PATH, {
		path: { invoice_id: invoiceId },
	}).queryKey
}

function invoicePaymentsKey(invoiceId: string) {
	return window.tanstackApi.get(INVOICE_PAYMENTS_PATH, {
		path: { invoice_id: invoiceId },
	}).queryKey
}

function invalidateInvoicesList(
	queryClient: ReturnType<typeof useQueryClient>,
	organizationId?: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isOrganizationQuery(INVOICES_PATH, query.queryKey, organizationId),
	})
}

function invalidateOutstanding(
	queryClient: ReturnType<typeof useQueryClient>,
	organizationId?: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isOrganizationQuery(OUTSTANDING_PATH, query.queryKey, organizationId),
	})
}

/**
 * What "what remains" depends on for one invoice: its own document, its
 * balance, its credit notes and its payments. Recording a payment, issuing
 * a credit note or deleting a payment can move any of these — the amount
 * saved by invalidating only the one that changed is not worth a screen
 * that shows a stale "remaining" next to a fresh payment list.
 */
function invalidateInvoiceDetail(
	queryClient: ReturnType<typeof useQueryClient>,
	invoiceId: string,
) {
	return Promise.all([
		queryClient.invalidateQueries({ queryKey: invoiceKey(invoiceId) }),
		queryClient.invalidateQueries({ queryKey: invoiceBalanceKey(invoiceId) }),
		queryClient.invalidateQueries({
			queryKey: invoiceCreditNotesKey(invoiceId),
		}),
		queryClient.invalidateQueries({
			queryKey: invoicePaymentsKey(invoiceId),
		}),
	])
}

/**
 * The organization's invoices, one page at a time.
 *
 * `listInvoices` only accepts `page`/`per_page` (see `handlers-invoice`'s
 * `invoice::list::handler`) — status and customer filtering happen
 * client-side, in the list UI, over whatever page is already in hand. That
 * is filtering a list of documents for display, not recomputing money, so
 * it does not fall under the backend-only pricing rule.
 */
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

/** What remains to be collected — computed backend-side, never here. */
export function useInvoiceBalance(invoiceId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(INVOICE_BALANCE_PATH, {
			path: { invoice_id: invoiceId },
		}).queryOptions,
		enabled: enabled && Boolean(invoiceId),
	})
}

export function useInvoiceCreditNotes(invoiceId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(INVOICE_CREDIT_NOTES_PATH, {
			path: { invoice_id: invoiceId },
		}).queryOptions,
		enabled: enabled && Boolean(invoiceId),
	})
}

export function useInvoicePayments(invoiceId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(INVOICE_PAYMENTS_PATH, {
			path: { invoice_id: invoiceId },
		}).queryOptions,
		enabled: enabled && Boolean(invoiceId),
	})
}

/** One number per customer, not per invoice — backs the list's outstanding
 * total without an N+1 read over every row. */
export function useOutstandingByCustomer(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(OUTSTANDING_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCreateInvoice(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICES_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, organizationId),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
		meta: { organizationId },
	})
}

/** Only ever called on a draft (CLAUDE.md: an invoice is immutable once
 * issued) — no balance/payments/credit-notes to invalidate here. */
export function useUpdateInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', INVOICE_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, invoice.data.organization_id),
				queryClient.invalidateQueries({
					queryKey: invoiceKey(invoice.data.id),
				}),
			])
		},
	})
}

export function useIssueInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICE_ISSUE_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, invoice.data.organization_id),
				invalidateOutstanding(queryClient, invoice.data.organization_id),
				invalidateInvoiceDetail(queryClient, invoice.data.id),
			])
		},
	})
}

export function useCancelInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICE_CANCEL_PATH).mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, invoice.data.organization_id),
				invalidateOutstanding(queryClient, invoice.data.organization_id),
				invalidateInvoiceDetail(queryClient, invoice.data.id),
			])
		},
	})
}

/** The response is the credit note itself — a new invoice document — but
 * what an artisan is looking at when they issue one is the source invoice,
 * so this invalidates the source's detail (via `variables.path.invoice_id`)
 * rather than the credit note's own, brand-new one. */
export function useIssueCreditNote() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICE_CREDIT_NOTES_PATH)
			.mutationOptions,
		onSuccess: async (creditNote, variables) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, creditNote.data.organization_id),
				invalidateOutstanding(queryClient, creditNote.data.organization_id),
				invalidateInvoiceDetail(queryClient, variables.path.invoice_id),
			])
		},
	})
}

export function useRecordPayment() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVOICE_PAYMENTS_PATH)
			.mutationOptions,
		onSuccess: async (payment) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, payment.data.organization_id),
				invalidateOutstanding(queryClient, payment.data.organization_id),
				invalidateInvoiceDetail(queryClient, payment.data.invoice_id),
			])
		},
	})
}

/**
 * `DELETE /invoice-payments/{invoice_payment_id}` carries no invoice id of
 * its own (see `paths::InvoicePaymentPath`'s own "bare" doc comment) — the
 * caller already knows it, from the invoice page it is calling this from,
 * same reason `useDeleteQuote` takes `organizationId` as a hook argument
 * rather than reading it off the mutation's response.
 */
export function useDeletePayment(invoiceId: string, organizationId?: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', INVOICE_PAYMENT_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				invalidateInvoicesList(queryClient, organizationId),
				invalidateOutstanding(queryClient, organizationId),
				invalidateInvoiceDetail(queryClient, invoiceId),
			])
		},
	})
}

/**
 * Project-scoped issuance built now because it lives in this same file, not
 * because #321 has a screen for it yet — #322 ("invoice a project") is the
 * first caller. Invalidates the organization's invoice list and outstanding
 * total the same way the invoice-scoped issuing hooks do.
 */
export function useIssueDeposit() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_INVOICE_DEPOSIT_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, invoice.data.organization_id),
				invalidateOutstanding(queryClient, invoice.data.organization_id),
			])
		},
	})
}

export function useIssueFinalInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_INVOICE_FINAL_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateInvoicesList(queryClient, invoice.data.organization_id),
				invalidateOutstanding(queryClient, invoice.data.organization_id),
			])
		},
	})
}

export type Invoice = Schemas.InvoiceResponse
export type InvoiceLine = Schemas.InvoiceLineResponse
export type InvoicePayment = Schemas.InvoicePaymentResponse
export type InvoiceBalance = Schemas.InvoiceBalanceResponse
export type CustomerOutstandingBalance =
	Schemas.CustomerOutstandingBalanceResponse
export type InvoiceStatus = Schemas.InvoiceStatus
export type InvoiceKind = Schemas.InvoiceKind
export type OperationNature = Schemas.OperationNature
export type InvoiceLinePayload = Schemas.InvoiceLineRequest
export type CreateInvoicePayload = Schemas.CreateInvoiceRequest
export type UpdateInvoicePayload = Schemas.UpdateInvoiceRequest
export type IssueCreditNotePayload = Schemas.IssueCreditNoteRequest
export type RecordInvoicePaymentPayload = Schemas.RecordInvoicePaymentRequest
export type IssueDepositPayload = Schemas.IssueDepositRequest
export type IssueFinalInvoicePayload = Schemas.IssueFinalInvoiceRequest
export type PaginationMetadata = Schemas.PaginationMetadata
