import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const LEDGER_PATH = '/api/v1/organizations/{organization_id}/ledger'
const EXPENSES_PATH = '/api/v1/organizations/{organization_id}/expenses'
const EXPENSE_PATH = '/api/v1/expenses/{expense_id}'
const EXPENSE_PAY_PATH = '/api/v1/expenses/{expense_id}/pay'
const EXPENSE_REIMBURSE_PATH = '/api/v1/expenses/{expense_id}/reimburse'
const CATEGORIES_PATH =
	'/api/v1/organizations/{organization_id}/expense-categories'
const RECEIPTS_PATH = '/api/v1/organizations/{organization_id}/receipts'
const RECEIPT_ALLOCATE_PATH = '/api/v1/receipts/{receipt_id}/allocate'
const INVOICES_PATH = '/api/v1/organizations/{organization_id}/invoices'
const OUTSTANDING_PATH =
	'/api/v1/organizations/{organization_id}/invoices/outstanding'

export type Ledger = Schemas.LedgerResponse
export type LedgerEntryRow = Schemas.LedgerEntryResponse
export type ExpenseRow = Schemas.ExpenseResponse
export type ExpenseCategoryRow = Schemas.ExpenseCategoryResponse
export type ExpenseBody = Schemas.CreateExpenseRequest
export type ExpenseUpdateBody = Schemas.UpdateExpenseRequest
export type ReceiptBody = Schemas.RecordReceiptRequest

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function useTreasuryInvalidation() {
	const queryClient = useQueryClient()
	const paths: readonly string[] = [
		LEDGER_PATH,
		EXPENSES_PATH,
		EXPENSE_PATH,
		CATEGORIES_PATH,
		INVOICES_PATH,
		OUTSTANDING_PATH,
	]

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => {
				const id = queryKeyMeta(query.queryKey)?._id
				return typeof id === 'string' && paths.includes(id)
			},
		})
}

export function useLedgerRead(organizationId: string, month: string | null) {
	return useQuery({
		...window.tanstackApi.get(LEDGER_PATH, {
			path: { organization_id: organizationId },
			query: month === null ? {} : { month },
		}).queryOptions,
		enabled: Boolean(organizationId),
		placeholderData: (previous) => previous,
	})
}

export function useExpenses(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(EXPENSES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useExpenseCategories(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(CATEGORIES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCreateExpense() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', EXPENSES_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useUpdateExpense() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('patch', EXPENSE_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useDeleteExpense() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('delete', EXPENSE_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function usePayExpense() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', EXPENSE_PAY_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useReimburseExpense() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', EXPENSE_REIMBURSE_PATH)
			.mutationOptions,
		onSuccess: invalidate,
	})
}

export function useRecordReceipt() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', RECEIPTS_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useAllocateReceipt() {
	const invalidate = useTreasuryInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', RECEIPT_ALLOCATE_PATH)
			.mutationOptions,
		onSuccess: invalidate,
	})
}
