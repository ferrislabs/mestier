import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	apiMethodOf,
	categoryOf,
	entryOf,
	expenseBody,
	expenseOf,
	methodOf,
	openInvoicesOf,
	proofLabel,
	receiptOf,
	summaryOf,
	trendOf,
	vatOf,
} from '#/pages/treasury/lib/ledger-api'
import { emptyExpenseValues } from '#/pages/treasury/lib/ledger-model'

function expenseRow(
	overrides: Partial<Schemas.ExpenseResponse> = {},
): Schemas.ExpenseResponse {
	return {
		id: 'e1',
		organization_id: 'o1',
		date: '2026-11-01',
		label: 'Quincaillerie',
		category_id: 'c1',
		amount_ht_cents: 84200,
		vat_cents: 16840,
		total_cents: 101040,
		status: 'TO_PAY',
		display_state: 'OVERDUE',
		days_late: 3,
		due_date: '2026-11-04',
		created_at: '2026-11-01T00:00:00Z',
		updated_at: '2026-11-01T00:00:00Z',
		supplier_name: 'Bernard',
		...overrides,
	}
}

function entryRow(
	overrides: Partial<Schemas.LedgerEntryResponse> = {},
): Schemas.LedgerEntryResponse {
	return {
		key: 'receipt:r1',
		kind: 'RECEIPT',
		source_id: 'r1',
		date: '2026-11-02',
		label: '',
		party: 'Lemoine',
		reference: 'VIR 1105',
		amount_cents: 50000,
		status: 'DONE',
		method: 'TRANSFER',
		invoice_id: 'i1',
		invoice_number: 'F-2026-109',
		...overrides,
	}
}

describe('payment methods', () => {
	it('converts between the API and the screen forms', () => {
		expect(methodOf('DIRECT_DEBIT')).toBe('direct_debit')
		expect(apiMethodOf('card')).toBe('CARD')
	})
})

describe('expenseOf', () => {
	it('carries the total, the state and the days late from the API', () => {
		const expense = expenseOf(expenseRow())

		expect(expense.total).toBe(101040)
		expect(expense.state).toBe('overdue')
		expect(expense.daysLate).toBe(3)
		expect(expense.paid).toBe(false)
		expect(expense.supplierName).toBe('Bernard')
		expect(expense.reference).toBe('')
		expect(expense.dueDate).toBe('2026-11-04')
	})

	it('reads a paid and advanced expense', () => {
		const expense = expenseOf(
			expenseRow({
				status: 'PAID',
				display_state: 'TO_REIMBURSE',
				paid_on: '2026-11-01',
				method: 'CARD',
				advanced_by_member_id: 'm1',
				due_date: null,
				proof_file_key: 'proofs/ticket.jpg',
			}),
		)

		expect(expense.paid).toBe(true)
		expect(expense.method).toBe('card')
		expect(expense.advancedById).toBe('m1')
		expect(expense.state).toBe('to_reimburse')
		expect(proofLabel(expense.proofName)).toBe('ticket.jpg')
		expect(proofLabel(null)).toBeNull()
	})
})

describe('ledger entries', () => {
	it('labels a payment, a refund and an unallocated receipt', () => {
		expect(entryOf(entryRow()).label).toBe('Paiement F-2026-109')
		expect(entryOf(entryRow({ amount_cents: -2000 })).label).toBe(
			'Remboursement F-2026-109',
		)
		const unallocated = entryOf(
			entryRow({
				invoice_id: null,
				invoice_number: null,
				status: 'UNALLOCATED',
				label: 'Acompte Fabre',
			}),
		)
		expect(unallocated.label).toBe('Acompte Fabre')
		expect(unallocated.status).toBe('unallocated')
		expect(entryOf(entryRow({ invoice_number: null, label: '' })).label).toBe(
			'Recette non affectée',
		)
	})

	it('keeps the raw note on the receipt detail', () => {
		const receipt = receiptOf(
			entryRow({ invoice_number: null, label: 'Acompte Fabre' }),
		)

		expect(receipt.note).toBe('Acompte Fabre')
		expect(receipt.invoiceLabel).toBeNull()
		expect(receipt.id).toBe('r1')
	})

	it('maps an expense entry with its category and a negative amount', () => {
		const entry = entryOf(
			entryRow({
				key: 'expense:e1',
				kind: 'EXPENSE',
				source_id: 'e1',
				label: 'Quincaillerie',
				amount_cents: -101040,
				status: 'LATE',
				category_id: 'c1',
				invoice_id: null,
				invoice_number: null,
			}),
		)

		expect(entry.kind).toBe('expense')
		expect(entry.amount).toBe(-101040)
		expect(entry.status).toBe('late')
		expect(entry.categoryId).toBe('c1')
		expect(entry.origin).toBe('manual')
	})
})

describe('read model figures', () => {
	it('renames the summary, the trend and the VAT without computing', () => {
		expect(
			summaryOf({
				receipts_cents: 1,
				expenses_cents: 2,
				net_cents: -1,
				pending_cents: 3,
				unallocated_cents: 4,
			}),
		).toEqual({
			receipts: 1,
			expenses: 2,
			net: -1,
			pendingExpenses: 3,
			unallocated: 4,
		})
		expect(
			trendOf([{ month: '2026-11', receipts_cents: 5, expenses_cents: 6 }]),
		).toEqual([{ month: '2026-11', receipts: 5, expenses: 6 }])
		expect(
			vatOf({
				collected_cents: 20000,
				deductible_cents: 2000,
				due_cents: 18000,
			}),
		).toEqual({ collected: 20000, deductible: 2000, due: 18000 })
	})

	it('maps a category', () => {
		expect(
			categoryOf({
				id: 'c1',
				organization_id: 'o1',
				name: 'Matériaux',
				hue: 60,
				created_at: '',
				updated_at: '',
			}),
		).toEqual({ id: 'c1', name: 'Matériaux', hue: 60 })
	})
})

describe('openInvoicesOf', () => {
	const invoice = (
		overrides: Partial<Schemas.InvoiceResponse>,
	): Schemas.InvoiceResponse =>
		({
			id: 'i1',
			number: 'F-2026-109',
			kind: 'STANDARD',
			customer_id: 'c1',
			balance_cents: 40000,
			display_state: 'UNPAID',
			...overrides,
		}) as Schemas.InvoiceResponse

	it('keeps issued invoices with a balance and drops the others', () => {
		const rows = openInvoicesOf(
			[
				invoice({}),
				invoice({ id: 'i2', display_state: 'PAID', balance_cents: 0 }),
				invoice({ id: 'i3', display_state: 'DRAFT', number: null }),
				invoice({ id: 'i4', kind: 'CREDIT_NOTE' }),
				invoice({ id: 'i5', display_state: 'OVERDUE' }),
			],
			(customerId) => (customerId === 'c1' ? 'Lemoine' : ''),
		)

		expect(rows.map((row) => row.id)).toEqual(['i1', 'i5'])
		expect(rows[0]).toEqual({
			id: 'i1',
			label: 'F-2026-109',
			customerName: 'Lemoine',
			balance: 40000,
		})
	})
})

describe('expenseBody', () => {
	const values = {
		...emptyExpenseValues('2026-11-01'),
		label: ' Carburant ',
		categoryId: 'c1',
		amountHt: '96',
		vatAmount: '19,20',
		reference: ' ',
		dueDate: '2026-11-30',
		note: '',
	}

	it('sends cents and trimmed optional texts', () => {
		expect(expenseBody(values, null)).toEqual({
			date: '2026-11-01',
			label: 'Carburant',
			supplier_id: null,
			category_id: 'c1',
			reference: null,
			amount_ht_cents: 9600,
			vat_cents: 1920,
			due_date: '2026-11-30',
			advanced_by_member_id: null,
			customer_id: null,
			proof_file_key: null,
			note: null,
		})
	})

	it('drops the due date of an advanced expense and defaults a blank VAT to zero', () => {
		const body = expenseBody({ ...values, vatAmount: '' }, 'm1')

		expect(body.due_date).toBeNull()
		expect(body.advanced_by_member_id).toBe('m1')
		expect(body.vat_cents).toBe(0)
	})
})
