import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers, useUploadFile } from '#/hooks/use-customers'
import {
	useOutstandingByCustomer,
	useRecordPayment,
} from '#/hooks/use-invoices'
import { useSuppliers } from '#/hooks/use-suppliers'
import {
	useAllocateReceipt,
	useCreateExpense,
	useDeleteExpense,
	useExpenseCategories,
	useExpenses,
	useLedgerRead,
	usePayExpense,
	useRecordReceipt,
	useReimburseExpense,
	useUpdateExpense,
} from '#/hooks/use-treasury'
import { mutationErrorMessage } from '#/lib/api-error'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { useOrganizationPeople } from '#/pages/customers/feature/use-organization-people'
import { parseAmount, todayIso } from '#/pages/customers/lib/format'
import { findPerson } from '#/pages/customers/lib/people'
import { useOpenInvoices } from '#/pages/treasury/feature/use-open-invoices'
import {
	apiMethodOf,
	categoryOf,
	entryOf,
	expenseBody,
	expenseOf,
	receiptOf,
	summaryOf,
	trendOf,
	vatOf,
} from '#/pages/treasury/lib/ledger-api'
import {
	type ExpenseFormValues,
	filterLedger,
	type LedgerSearch,
	type LedgerSearchPatch,
	ledgerFiltersOf,
	type PaymentFormValues,
} from '#/pages/treasury/lib/ledger-model'
import { LedgerUI } from '#/pages/treasury/ui/ledger-ui'

interface LedgerFeatureProps {
	search: LedgerSearch
	onSearchChange: (patch: LedgerSearchPatch) => void
}

const EMPTY_SUMMARY = {
	receipts: 0,
	expenses: 0,
	net: 0,
	pendingExpenses: 0,
	unallocated: 0,
}

const EMPTY_VAT = { collected: 0, deductible: 0, due: 0 }

export function LedgerFeature({ search, onSearchChange }: LedgerFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const today = todayIso()
	const notice = useNotice()
	const filters = ledgerFiltersOf(search)
	const ledger = useLedgerRead(organizationId, filters.month).data?.data
	const expenseRows = useExpenses(organizationId).data?.data
	const categoryRows = useExpenseCategories(organizationId).data?.data
	const openInvoices = useOpenInvoices(organizationId)
	const outstanding = useOutstandingByCustomer(organizationId)
	const suppliers = useSuppliers(organizationId)
	const customers = useCustomers(organizationId, { page: 1, perPage: 100 })
	const { people } = useOrganizationPeople(organizationId)
	const uploadFile = useUploadFile()
	const createExpense = useCreateExpense()
	const updateExpense = useUpdateExpense()
	const deleteExpense = useDeleteExpense()
	const payExpense = usePayExpense()
	const reimburseExpense = useReimburseExpense()
	const recordReceipt = useRecordReceipt()
	const recordPayment = useRecordPayment()
	const allocateReceipt = useAllocateReceipt()

	const entries = useMemo(() => (ledger?.entries ?? []).map(entryOf), [ledger])
	const receipts = useMemo(
		() =>
			(ledger?.entries ?? [])
				.filter((entry) => entry.kind === 'RECEIPT')
				.map(receiptOf),
		[ledger],
	)
	const expenses = useMemo(
		() => (expenseRows ?? []).map(expenseOf),
		[expenseRows],
	)
	const categories = useMemo(
		() => (categoryRows ?? []).map(categoryOf),
		[categoryRows],
	)
	const customerRows = (customers.data?.data ?? []).filter(
		(customer) => customer.status !== 'ARCHIVED',
	)
	const visible = filterLedger(entries, filters)
	const balances = (outstanding.data?.data ?? []).filter(
		(row) => row.outstanding_cents > 0,
	)
	const toCollect = balances.reduce(
		(sum, row) => sum + row.outstanding_cents,
		0,
	)
	const supplierRows = (suppliers.data?.data ?? []).map((supplier) => ({
		id: supplier.id,
		name: supplier.name,
	}))

	function fail(error: unknown) {
		notice.show(mutationErrorMessage(error) ?? 'Action impossible.')
		throw error
	}

	async function saveExpenseValues(
		expenseId: string | null,
		values: ExpenseFormValues,
	) {
		const existing = expenses.find((item) => item.id === expenseId)
		const advanced = existing ? existing.advancedById : values.advancedById
		const { advanced_by_member_id: _advanced, ...body } = expenseBody(
			values,
			advanced,
		)
		try {
			if (expenseId === null)
				await createExpense.mutateAsync({
					path: { organization_id: organizationId },
					body: expenseBody(values, advanced),
				})
			else
				await updateExpense.mutateAsync({
					path: { expense_id: expenseId },
					body,
				})
		} catch (error) {
			return fail(error)
		}
		notice.show(expenseId ? 'Dépense mise à jour.' : 'Dépense enregistrée.')
	}

	async function recordReceiptValues(values: PaymentFormValues) {
		const amount = parseAmount(values.amount) ?? 0
		const reference = values.reference.trim() || null
		const note = values.note.trim() || null
		try {
			if (values.invoiceId !== null)
				await recordPayment.mutateAsync({
					path: { invoice_id: values.invoiceId },
					body: {
						amount_cents: amount,
						paid_on: values.date,
						method: apiMethodOf(values.method),
						reference,
						note,
					},
				})
			else
				await recordReceipt.mutateAsync({
					path: { organization_id: organizationId },
					body: {
						customer_id: values.customerId,
						amount_cents: amount,
						paid_on: values.date,
						method: apiMethodOf(values.method),
						reference,
						note,
					},
				})
		} catch (error) {
			return fail(error)
		}
		notice.show('Recette enregistrée.')
	}

	return (
		<LedgerUI
			organizationSlug={activeOrganization.slug}
			today={today}
			search={search}
			onSearchChange={onSearchChange}
			notice={notice.message}
			onDismissNotice={notice.clear}
			entries={entries}
			visible={visible}
			period={ledger ? summaryOf(ledger.summary) : EMPTY_SUMMARY}
			periodMonth={filters.month}
			toCollect={toCollect}
			toCollectCount={balances.length}
			toPay={ledger?.payables.total_cents ?? 0}
			overdueToPay={ledger?.payables.overdue_cents ?? 0}
			vat={ledger ? vatOf(ledger.vat) : EMPTY_VAT}
			trend={ledger ? trendOf(ledger.trend) : []}
			months={ledger?.months ?? []}
			categories={categories}
			expenses={expenses}
			receipts={receipts}
			openInvoices={openInvoices}
			suppliers={supplierRows}
			customers={customerRows}
			people={people}
			onUploadProof={async (file) => {
				const uploaded = await uploadFile.mutateAsync(file)
				return uploaded.data.key
			}}
			detailActions={{
				onDeleteExpense: (expense) => {
					void deleteExpense
						.mutateAsync({ path: { expense_id: expense.id } })
						.then(() => notice.show('Dépense supprimée.'))
						.catch((error) =>
							notice.show(mutationErrorMessage(error) ?? 'Action impossible.'),
						)
				},
				onMarkPaid: (expense, paidAt, method) => {
					void payExpense
						.mutateAsync({
							path: { expense_id: expense.id },
							body: { paid_on: paidAt, method: apiMethodOf(method) },
						})
						.then(() => notice.show('Dépense marquée comme payée.'))
						.catch((error) =>
							notice.show(mutationErrorMessage(error) ?? 'Action impossible.'),
						)
				},
				onReimburse: (expense) => {
					void reimburseExpense
						.mutateAsync({ path: { expense_id: expense.id } })
						.then(() =>
							notice.show(
								`Remboursement de ${findPerson(people, expense.advancedById)?.name ?? 'la personne'} enregistré.`,
							),
						)
						.catch((error) =>
							notice.show(mutationErrorMessage(error) ?? 'Action impossible.'),
						)
				},
				onAllocate: (receipt, invoice) => {
					void allocateReceipt
						.mutateAsync({
							path: { receipt_id: receipt.id },
							body: { invoice_id: invoice.id },
						})
						.then(() => notice.show(`Recette affectée à ${invoice.label}.`))
						.catch((error) =>
							notice.show(mutationErrorMessage(error) ?? 'Action impossible.'),
						)
				},
			}}
			onSaveExpense={saveExpenseValues}
			onRecordReceipt={recordReceiptValues}
		/>
	)
}
