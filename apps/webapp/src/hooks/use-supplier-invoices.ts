import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const SUPPLIER_INVOICES_PATH =
	'/api/v1/organizations/{organization_id}/supplier-invoices'
const SUPPLIER_INVOICE_IMPORT_PATH =
	'/api/v1/organizations/{organization_id}/supplier-invoices/import'
const SUPPLIER_INVOICE_PATH = '/api/v1/supplier-invoices/{supplier_invoice_id}'
const SUPPLIER_INVOICE_CONFIRM_PATH =
	'/api/v1/supplier-invoices/{supplier_invoice_id}/confirm'
const SUPPLIER_INVOICE_REJECT_PATH =
	'/api/v1/supplier-invoices/{supplier_invoice_id}/reject'
const LINE_ALLOCATIONS_PATH =
	'/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations'
const PROJECT_SUPPLIER_COSTS_PATH = '/api/v1/projects/{project_id}/supplier-costs'

interface SupplierInvoiceListParams {
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

function supplierInvoiceKey(supplierInvoiceId: string) {
	return window.tanstackApi.get(SUPPLIER_INVOICE_PATH, {
		path: { supplier_invoice_id: supplierInvoiceId },
	}).queryKey
}

function projectSupplierCostsKey(projectId: string) {
	return window.tanstackApi.get(PROJECT_SUPPLIER_COSTS_PATH, {
		path: { project_id: projectId },
	}).queryKey
}

function invalidateSupplierInvoicesList(
	queryClient: ReturnType<typeof useQueryClient>,
	organizationId?: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isOrganizationQuery(SUPPLIER_INVOICES_PATH, query.queryKey, organizationId),
	})
}

function invalidateSupplierInvoiceDetail(
	queryClient: ReturnType<typeof useQueryClient>,
	supplierInvoiceId: string,
) {
	return queryClient.invalidateQueries({
		queryKey: supplierInvoiceKey(supplierInvoiceId),
	})
}

/** Every non-deleted supplier invoice for the organization, oldest first —
 * see `handlers-purchase`'s own `list::handler` for the ordering. */
export function useSupplierInvoices(
	organizationId: string,
	params: SupplierInvoiceListParams = { page: 1, perPage: 100 },
) {
	return useQuery(
		window.tanstackApi.get(SUPPLIER_INVOICES_PATH, {
			path: { organization_id: organizationId },
			query: { page: params.page, per_page: params.perPage },
		}).queryOptions,
	)
}

export function useSupplierInvoice(supplierInvoiceId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(SUPPLIER_INVOICE_PATH, {
			path: { supplier_invoice_id: supplierInvoiceId },
		}).queryOptions,
		enabled: enabled && Boolean(supplierInvoiceId),
	})
}

/**
 * Uploads and parses a file in one call (#339: the file is stored, not only
 * parsed). Not `window.tanstackApi.mutation`: the request body is the raw
 * file, not JSON, same reason `useUploadFile` (`use-customers.ts`) bypasses
 * it too. A parse failure is a `200` carrying `outcome: "parse_failed"`, not
 * a thrown error — the caller reads `.data.outcome`, it never rejects on
 * that account.
 */
export function useImportSupplierInvoice(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		mutationFn: async (file: File) => {
			const body = await file.arrayBuffer()
			return window.api.post(SUPPLIER_INVOICE_IMPORT_PATH, {
				path: { organization_id: organizationId },
				body,
				header: {
					'Content-Type': file.type || 'application/octet-stream',
				},
			} as never) as Promise<{
				data: Schemas.ImportSupplierInvoiceResponse
			}>
		},
		onSuccess: async (result) => {
			if (result.data.outcome !== 'created') return
			await invalidateSupplierInvoicesList(queryClient, organizationId)
		},
	})
}

/** Notes only — the document's own fields are somebody else's facts and are
 * never editable here, at any status. */
export function useUpdateSupplierInvoiceNotes() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', SUPPLIER_INVOICE_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateSupplierInvoicesList(queryClient, invoice.data.organization_id),
				invalidateSupplierInvoiceDetail(queryClient, invoice.data.id),
			])
		},
	})
}

export function useConfirmSupplierInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', SUPPLIER_INVOICE_CONFIRM_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateSupplierInvoicesList(queryClient, invoice.data.organization_id),
				invalidateSupplierInvoiceDetail(queryClient, invoice.data.id),
			])
		},
	})
}

export function useRejectSupplierInvoice() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', SUPPLIER_INVOICE_REJECT_PATH)
			.mutationOptions,
		onSuccess: async (invoice) => {
			await Promise.all([
				invalidateSupplierInvoicesList(queryClient, invoice.data.organization_id),
				invalidateSupplierInvoiceDetail(queryClient, invoice.data.id),
			])
		},
	})
}

/**
 * Full-replace of one line's allocations (#339, mirroring `Task::assignments`'s
 * own `PUT`). Invalidates every project any of the *previous* allocations
 * pointed at as well as the new ones: a share moved off a project must stop
 * counting on that project's own read, not just start counting on the new
 * one.
 */
export function useReplaceLineAllocations(previousProjectIds: string[] = []) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', LINE_ALLOCATIONS_PATH).mutationOptions,
		onSuccess: async (allocations) => {
			const projectIds = new Set(previousProjectIds)
			for (const allocation of allocations.data) {
				projectIds.add(allocation.project_id)
			}
			await Promise.all(
				Array.from(projectIds, (projectId) =>
					queryClient.invalidateQueries({
						queryKey: projectSupplierCostsKey(projectId),
					}),
				),
			)
		},
	})
}

/** What a project's supplier costs look like on its own screen — a total
 * plus the itemized lines that add up to it, each pointing back at the
 * invoice it came from (#340). */
export function useProjectSupplierCosts(projectId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_SUPPLIER_COSTS_PATH, {
			path: { project_id: projectId },
		}).queryOptions,
		enabled: enabled && Boolean(projectId),
	})
}

export type SupplierInvoice = Schemas.SupplierInvoiceResponse
export type SupplierInvoiceLine = Schemas.SupplierInvoiceLineResponse
export type SupplierInvoiceStatus = Schemas.SupplierInvoiceStatus
export type SupplierInvoiceSource = Schemas.SupplierInvoiceSource
export type ImportSupplierInvoiceOutcome = Schemas.ImportSupplierInvoiceResponse
export type SupplierInvoiceLineAllocation =
	Schemas.SupplierInvoiceLineAllocationResponse
export type ReplaceLineAllocationsPayload = Schemas.ReplaceLineAllocationsRequest
export type LineAllocationShare = Schemas.LineAllocationShareRequest
export type ProjectSupplierCosts = Schemas.ProjectSupplierCostsResponse
export type ProjectSupplierCostLine = Schemas.ProjectSupplierCostLineResponse
export type PaginationMetadata = Schemas.PaginationMetadata
