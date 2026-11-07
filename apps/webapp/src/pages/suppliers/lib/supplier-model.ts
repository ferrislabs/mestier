import type { Supplier } from '#/hooks/use-suppliers'
import { normalizeText } from '#/pages/catalog/catalog-model'
import { addDays } from '#/pages/customers/lib/format'
import { EMPTY_SUPPLIER_EXTRAS } from '#/pages/suppliers/feature/mocks/supplier-extras'
import { contactName } from '#/pages/suppliers/lib/supplier-form'
import type { SupplierExtras } from '#/pages/suppliers/types'
import { expenseState, expenseTotal } from '#/pages/treasury/lib/ledger-model'
import type { Expense, ExpenseCategory } from '#/pages/treasury/types'

export interface SupplierRow {
	supplier: Supplier
	extras: SupplierExtras
	contact: string
	contactEmail: string
	category: ExpenseCategory | null
	spend12m: number
	toPay: number
	overdue: boolean
}

export interface SupplierRowFilters {
	archived?: boolean
	query?: string
}

interface SupplierRowsInput {
	suppliers: readonly Supplier[]
	extras: Readonly<Record<string, SupplierExtras>>
	expenses: readonly Expense[]
	categories: readonly ExpenseCategory[]
	today: string
	filters: SupplierRowFilters
}

function rowOf(
	supplier: Supplier,
	extras: SupplierExtras,
	expenses: readonly Expense[],
	categories: readonly ExpenseCategory[],
	today: string,
): SupplierRow {
	const since = addDays(today, -365)
	const mine = expenses.filter((expense) => expense.supplierId === supplier.id)
	const unpaid = mine.filter((expense) => {
		const state = expenseState(expense, today)
		return state === 'to_pay' || state === 'overdue'
	})
	const contact = extras.contacts?.at(0)
	return {
		supplier,
		extras,
		contact: contact ? contactName(contact) : '',
		contactEmail: contact?.email || (supplier.email ?? ''),
		category:
			categories.find((item) => item.id === extras.defaultCategoryId) ?? null,
		spend12m: mine
			.filter((expense) => expense.date >= since)
			.reduce((sum, expense) => sum + expenseTotal(expense), 0),
		toPay: unpaid.reduce((sum, expense) => sum + expenseTotal(expense), 0),
		overdue: unpaid.some(
			(expense) => expenseState(expense, today) === 'overdue',
		),
	}
}

function searchText(row: SupplierRow): string {
	return normalizeText(
		[
			row.supplier.name,
			row.supplier.registration_number ?? '',
			row.supplier.vat_number ?? '',
			row.supplier.email ?? '',
			row.supplier.phone ?? '',
			row.supplier.address ?? '',
			row.contact,
			row.contactEmail,
			row.category?.name ?? '',
		].join(' '),
	)
}

export function supplierRows(input: SupplierRowsInput): SupplierRow[] {
	const query = normalizeText(input.filters.query ?? '')
	const archived = input.filters.archived === true
	return input.suppliers
		.map((supplier) =>
			rowOf(
				supplier,
				input.extras[supplier.id] ?? EMPTY_SUPPLIER_EXTRAS,
				input.expenses,
				input.categories,
				input.today,
			),
		)
		.filter((row) => row.extras.archived === archived)
		.filter((row) => query === '' || searchText(row).includes(query))
		.sort((a, b) => a.supplier.name.localeCompare(b.supplier.name, 'fr'))
}

export function supplierTotals(rows: readonly SupplierRow[]): {
	spend12m: number
	toPay: number
	overdue: boolean
} {
	return {
		spend12m: rows.reduce((sum, row) => sum + row.spend12m, 0),
		toPay: rows.reduce((sum, row) => sum + row.toPay, 0),
		overdue: rows.some((row) => row.overdue),
	}
}

export function formatLegalId(value: string): string {
	const digits = value.replace(/\s/g, '')
	if (/^\d{14}$/.test(digits))
		return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6, 9)} ${digits.slice(9)}`
	if (/^\d{9}$/.test(digits))
		return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6)}`
	return value
}
