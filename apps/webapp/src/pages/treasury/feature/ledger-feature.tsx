import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useOutstandingByCustomer } from '#/hooks/use-invoices'
import { useSuppliers } from '#/hooks/use-suppliers'
import { findPerson } from '#/pages/customers/feature/mocks/people'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { parseAmount, todayIso } from '#/pages/customers/lib/format'
import {
	useAllocateReceipt,
	useDeleteExpense,
	useLedger,
	useMarkExpensePaid,
	useRecordReceipt,
	useReimburseExpense,
	useSaveExpense,
} from '#/pages/treasury/feature/mocks/ledger'
import {
	DEFAULT_LEDGER_FILTERS,
	type ExpenseFormValues,
	expenseState,
	expenseTotal,
	filterLedger,
	type LedgerSearch,
	type LedgerSearchPatch,
	ledgerEntries,
	ledgerFiltersOf,
	ledgerMonths,
	monthlyTrend,
	type PaymentFormValues,
	payables,
	summarizeLedger,
	vatSummary,
} from '#/pages/treasury/lib/ledger-model'
import type { Expense } from '#/pages/treasury/types'
import { LedgerUI } from '#/pages/treasury/ui/ledger-ui'

interface LedgerFeatureProps {
	search: LedgerSearch
	onSearchChange: (patch: LedgerSearchPatch) => void
}

export function LedgerFeature({ search, onSearchChange }: LedgerFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const today = todayIso()
	const notice = useNotice()
	const ledger = useLedger().data
	const outstanding = useOutstandingByCustomer(activeOrganization.id)
	const suppliers = useSuppliers(activeOrganization.id)
	const customers = useCustomers(activeOrganization.id, {
		page: 1,
		perPage: 100,
	})
	const saveExpense = useSaveExpense()
	const deleteExpense = useDeleteExpense()
	const markPaid = useMarkExpensePaid()
	const reimburse = useReimburseExpense()
	const recordReceipt = useRecordReceipt()
	const allocate = useAllocateReceipt()

	const entries = useMemo(() => ledgerEntries(ledger, today), [ledger, today])
	const filters = ledgerFiltersOf(search)
	const visible = filterLedger(entries, filters)
	const period = summarizeLedger(
		filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, month: filters.month }),
	)
	const balances = (outstanding.data?.data ?? []).filter(
		(row) => row.outstanding_cents > 0,
	)
	const toCollect = balances.reduce(
		(sum, row) => sum + row.outstanding_cents,
		0,
	)
	const unpaid = payables(ledger, today)
	const toPay = unpaid.reduce((sum, expense) => sum + expenseTotal(expense), 0)
	const overdueToPay = unpaid
		.filter((expense) => expenseState(expense, today) === 'overdue')
		.reduce((sum, expense) => sum + expenseTotal(expense), 0)
	const trend = useMemo(() => monthlyTrend(entries, today), [entries, today])
	const months = useMemo(() => ledgerMonths(entries), [entries])
	const customerRows = (customers.data?.data ?? []).filter(
		(customer) => customer.status !== 'ARCHIVED',
	)
	const supplierRows = (suppliers.data?.data ?? []).map((supplier) => ({
		id: supplier.id,
		name: supplier.name,
	}))

	async function saveExpenseValues(
		expenseId: string | null,
		values: ExpenseFormValues,
	) {
		const existing = ledger.expenses.find((item) => item.id === expenseId)
		const advanced = existing ? existing.advancedById : values.advancedById
		const draft: Omit<Expense, 'id'> = {
			date: values.date,
			label: values.label.trim(),
			supplierId: values.supplierId,
			supplierName:
				supplierRows.find((supplier) => supplier.id === values.supplierId)
					?.name ?? '',
			categoryId: values.categoryId,
			reference: values.reference.trim(),
			amountHt: parseAmount(values.amountHt) ?? 0,
			vatAmount:
				values.vatAmount.trim() === ''
					? 0
					: (parseAmount(values.vatAmount) ?? 0),
			dueDate:
				advanced !== null || values.dueDate === '' ? null : values.dueDate,
			paid: existing ? existing.paid : advanced !== null,
			paidAt: existing
				? existing.paidAt
				: advanced !== null
					? values.date
					: null,
			method: existing ? existing.method : null,
			advancedById: advanced,
			reimbursedAt: existing ? existing.reimbursedAt : null,
			customerId: values.customerId,
			customerName:
				customerRows.find((customer) => customer.id === values.customerId)
					?.name ?? '',
			proofName: values.proofName,
			note: values.note.trim(),
		}
		await saveExpense.mutateAsync(expenseId, draft)
		notice.show(expenseId ? 'Dépense mise à jour.' : 'Dépense enregistrée.')
	}

	async function recordReceiptValues(values: PaymentFormValues) {
		const invoice = ledger.openInvoices.find(
			(item) => item.id === values.invoiceId,
		)
		const customer = customerRows.find((item) => item.id === values.customerId)
		const amount = parseAmount(values.amount) ?? 0
		await recordReceipt.mutateAsync({
			date: values.date,
			amount,
			vatAmount: Math.round((amount * 20) / 120),
			method: values.method,
			customerId: customer?.id ?? null,
			customerName: customer?.name ?? invoice?.customerName ?? '',
			invoiceLabel: invoice?.label ?? null,
			reference: values.reference.trim(),
			note: values.note.trim(),
		})
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
			period={period}
			periodMonth={filters.month}
			toCollect={toCollect}
			toCollectCount={balances.length}
			toPay={toPay}
			overdueToPay={overdueToPay}
			vat={vatSummary(ledger, filters.month)}
			trend={trend}
			months={months}
			categories={ledger.categories}
			expenses={ledger.expenses}
			receipts={ledger.receipts}
			openInvoices={ledger.openInvoices}
			suppliers={supplierRows}
			customers={customerRows}
			detailActions={{
				onDeleteExpense: (expense) => {
					void deleteExpense
						.mutateAsync(expense.id)
						.then(() => notice.show('Dépense supprimée.'))
				},
				onMarkPaid: (expense, paidAt, method) => {
					void markPaid
						.mutateAsync(expense.id, paidAt, method)
						.then(() => notice.show('Dépense marquée comme payée.'))
				},
				onReimburse: (expense) => {
					void reimburse
						.mutateAsync(expense.id, today)
						.then(() =>
							notice.show(
								`Remboursement de ${findPerson(expense.advancedById)?.name ?? 'la personne'} enregistré.`,
							),
						)
				},
				onAllocate: (receipt, invoice) => {
					void allocate
						.mutateAsync(receipt.id, invoice)
						.then(() => notice.show(`Recette affectée à ${invoice.label}.`))
				},
			}}
			onSaveExpense={saveExpenseValues}
			onRecordReceipt={recordReceiptValues}
		/>
	)
}
