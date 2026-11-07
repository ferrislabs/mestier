import type { Schemas } from '#/api/api.client'
import type {
	ExpenseBody,
	ExpenseCategoryRow,
	ExpenseRow,
	Ledger,
	LedgerEntryRow,
} from '#/hooks/use-treasury'
import { parseAmount } from '#/pages/customers/lib/format'
import type {
	ExpenseFormValues,
	LedgerSummary,
	MonthTrend,
	VatSummary,
} from '#/pages/treasury/lib/ledger-model'
import { basename } from '#/pages/treasury/lib/proof'
import type {
	Expense,
	ExpenseCategory,
	ExpenseDisplayState,
	LedgerEntry,
	LedgerStatus,
	OpenInvoice,
	PaymentMethod,
	Receipt,
} from '#/pages/treasury/types'

const METHODS: Record<Schemas.PaymentMethod, PaymentMethod> = {
	CARD: 'card',
	TRANSFER: 'transfer',
	CHECK: 'check',
	CASH: 'cash',
	DIRECT_DEBIT: 'direct_debit',
}

const API_METHODS: Record<PaymentMethod, Schemas.PaymentMethod> = {
	card: 'CARD',
	transfer: 'TRANSFER',
	check: 'CHECK',
	cash: 'CASH',
	direct_debit: 'DIRECT_DEBIT',
}

const STATES: Record<Schemas.ExpenseDisplayState, ExpenseDisplayState> = {
	TO_PAY: 'to_pay',
	OVERDUE: 'overdue',
	PAID: 'paid',
	TO_REIMBURSE: 'to_reimburse',
	REIMBURSED: 'reimbursed',
}

const STATUSES: Record<Schemas.LedgerStatus, LedgerStatus> = {
	DONE: 'done',
	PENDING: 'pending',
	LATE: 'late',
	UNALLOCATED: 'unallocated',
	TO_REIMBURSE: 'to_reimburse',
}

export function methodOf(method: Schemas.PaymentMethod): PaymentMethod {
	return METHODS[method]
}

export function apiMethodOf(method: PaymentMethod): Schemas.PaymentMethod {
	return API_METHODS[method]
}

export function categoryOf(row: ExpenseCategoryRow): ExpenseCategory {
	return { id: row.id, name: row.name, hue: row.hue }
}

export function expenseOf(row: ExpenseRow): Expense {
	return {
		id: row.id,
		date: row.date,
		label: row.label,
		supplierId: row.supplier_id ?? null,
		supplierName: row.supplier_name ?? '',
		categoryId: row.category_id,
		reference: row.reference ?? '',
		amountHt: row.amount_ht_cents,
		vatAmount: row.vat_cents,
		dueDate: row.due_date ?? null,
		paid: row.status === 'PAID',
		paidAt: row.paid_on ?? null,
		method: row.method ? methodOf(row.method) : null,
		advancedById: row.advanced_by_member_id ?? null,
		reimbursedAt: row.reimbursed_on ?? null,
		customerId: row.customer_id ?? null,
		customerName: row.customer_name ?? '',
		proofName: row.proof_file_key ?? null,
		note: row.note ?? '',
		total: row.total_cents,
		state: STATES[row.display_state],
		daysLate: row.days_late,
	}
}

function receiptLabel(row: LedgerEntryRow): string {
	if (row.invoice_number)
		return `${row.amount_cents < 0 ? 'Remboursement' : 'Paiement'} ${row.invoice_number}`
	return row.label || 'Recette non affectée'
}

export function entryOf(row: LedgerEntryRow): LedgerEntry {
	return {
		key: row.key,
		kind: row.kind === 'RECEIPT' ? 'receipt' : 'expense',
		sourceId: row.source_id,
		date: row.date,
		label: row.kind === 'RECEIPT' ? receiptLabel(row) : row.label,
		party: row.party,
		reference: row.reference,
		amount: row.amount_cents,
		status: STATUSES[row.status],
		method: row.method ? methodOf(row.method) : null,
		origin: 'manual',
		categoryId: row.category_id ?? null,
		invoiceLabel: row.invoice_number ?? null,
		customerId: row.customer_id ?? null,
	}
}

export function receiptOf(row: LedgerEntryRow): Receipt {
	return {
		id: row.source_id,
		date: row.date,
		amount: row.amount_cents,
		method: row.method ? methodOf(row.method) : 'transfer',
		source: 'manual',
		customerId: row.customer_id ?? null,
		customerName: row.party,
		invoiceLabel: row.invoice_number ?? null,
		reference: row.reference,
		note: row.label,
	}
}

export function summaryOf(summary: Ledger['summary']): LedgerSummary {
	return {
		receipts: summary.receipts_cents,
		expenses: summary.expenses_cents,
		net: summary.net_cents,
		pendingExpenses: summary.pending_cents,
		unallocated: summary.unallocated_cents,
	}
}

export function trendOf(trend: Ledger['trend']): MonthTrend[] {
	return trend.map((month) => ({
		month: month.month,
		receipts: month.receipts_cents,
		expenses: month.expenses_cents,
	}))
}

export function vatOf(vat: Ledger['vat']): VatSummary {
	return {
		collected: vat.collected_cents,
		deductible: vat.deductible_cents,
		due: vat.due_cents,
	}
}

export function openInvoicesOf(
	invoices: readonly Schemas.InvoiceResponse[],
	customerName: (customerId: string) => string,
): OpenInvoice[] {
	return invoices
		.filter(
			(invoice) =>
				invoice.kind !== 'CREDIT_NOTE' &&
				invoice.number != null &&
				invoice.balance_cents > 0 &&
				(invoice.display_state === 'UNPAID' ||
					invoice.display_state === 'PARTIAL' ||
					invoice.display_state === 'OVERDUE'),
		)
		.map((invoice) => ({
			id: invoice.id,
			label: invoice.number ?? '',
			customerName: customerName(invoice.customer_id),
			balance: invoice.balance_cents,
		}))
}

function optionalText(text: string): string | null {
	const trimmed = text.trim()
	return trimmed === '' ? null : trimmed
}

export function expenseBody(
	values: ExpenseFormValues,
	advancedById: string | null,
): ExpenseBody {
	return {
		date: values.date,
		label: values.label.trim(),
		supplier_id: values.supplierId,
		category_id: values.categoryId,
		reference: optionalText(values.reference),
		amount_ht_cents: parseAmount(values.amountHt) ?? 0,
		vat_cents:
			values.vatAmount.trim() === '' ? 0 : (parseAmount(values.vatAmount) ?? 0),
		due_date:
			advancedById !== null || values.dueDate === '' ? null : values.dueDate,
		advanced_by_member_id: advancedById,
		customer_id: values.customerId,
		proof_file_key: values.proofName,
		note: optionalText(values.note),
	}
}

export function proofLabel(key: string | null): string | null {
	return key === null ? null : basename(key)
}
