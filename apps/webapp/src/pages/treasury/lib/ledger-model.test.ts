import { describe, expect, it } from 'vitest'
import {
	applyVatRate,
	canMarkPaid,
	canReimburse,
	changeAmountHt,
	cleanLedgerSearch,
	DEFAULT_LEDGER_FILTERS,
	emptyExpenseValues,
	emptyPaymentValues,
	expenseFormErrors,
	expenseState,
	expenseTotal,
	filterLedger,
	ledgerEntries,
	ledgerFiltersOf,
	ledgerMonths,
	monthlyTrend,
	parseLedgerSearch,
	paymentFormErrors,
	summarizeLedger,
	validatePaidDate,
	vatSummary,
} from '#/pages/treasury/lib/ledger-model'
import type { Expense, LedgerState, Receipt } from '#/pages/treasury/types'

const TODAY = '2026-11-05'

function expense(overrides: Partial<Expense>): Expense {
	return {
		id: 'e1',
		date: '2026-11-01',
		label: 'Quincaillerie',
		supplierId: null,
		supplierName: 'Bernard',
		categoryId: 'cat-materiaux',
		reference: '',
		amountHt: 10000,
		vatAmount: 2000,
		dueDate: null,
		paid: true,
		paidAt: '2026-11-01',
		method: 'transfer',
		advancedById: null,
		reimbursedAt: null,
		customerId: null,
		customerName: '',
		proofName: null,
		note: '',
		...overrides,
	}
}

function receipt(overrides: Partial<Receipt>): Receipt {
	return {
		id: 'r1',
		date: '2026-11-02',
		amount: 50000,
		vatAmount: 8333,
		method: 'transfer',
		source: 'manual',
		customerId: null,
		customerName: 'Lemoine',
		invoiceLabel: 'F-2026-109',
		reference: '',
		note: '',
		...overrides,
	}
}

const state: LedgerState = {
	categories: [],
	openInvoices: [],
	expenses: [
		expense({ id: 'e1' }),
		expense({
			id: 'e2',
			date: '2026-10-10',
			paid: false,
			dueDate: '2026-10-30',
			paidAt: null,
		}),
		expense({ id: 'e3', advancedById: 'julien' }),
	],
	receipts: [
		receipt({ id: 'r1' }),
		receipt({ id: 'r2', invoiceLabel: null, date: '2026-10-05' }),
	],
}

describe('expenseState', () => {
	it('distinguishes paid, to pay, overdue and advanced expenses', () => {
		expect(expenseState(state.expenses[0] as Expense, TODAY)).toBe('paid')
		expect(expenseState(state.expenses[1] as Expense, TODAY)).toBe('overdue')
		expect(
			expenseState(expense({ paid: false, dueDate: '2026-12-01' }), TODAY),
		).toBe('to_pay')
		expect(expenseState(state.expenses[2] as Expense, TODAY)).toBe(
			'to_reimburse',
		)
		expect(
			expenseState(expense({ advancedById: 'a', reimbursedAt: TODAY }), TODAY),
		).toBe('reimbursed')
	})

	it('offers the right action', () => {
		expect(canMarkPaid(expense({ paid: false }))).toBe(true)
		expect(canMarkPaid(expense({ advancedById: 'a' }))).toBe(false)
		expect(canReimburse(expense({ advancedById: 'a' }))).toBe(true)
		expect(expenseTotal(expense({}))).toBe(12000)
	})

	it('validates a payment date', () => {
		const e = expense({ date: '2026-11-01' })
		expect(validatePaidDate(e, '2026-11-03', TODAY)).toEqual([])
		expect(validatePaidDate(e, '2026-12-01', TODAY)).toHaveLength(1)
		expect(validatePaidDate(e, '2026-10-01', TODAY)).toHaveLength(1)
		expect(validatePaidDate(e, '', TODAY)).toHaveLength(1)
	})
})

describe('ledger entries', () => {
	const entries = ledgerEntries(state, TODAY)

	it('sorts newest first and signs expenses negative', () => {
		expect(entries.map((entry) => entry.key)[0]).toBe('receipt:r1')
		expect(entries.find((entry) => entry.key === 'expense:e1')?.amount).toBe(
			-12000,
		)
	})

	it('flags an unallocated receipt and a late expense', () => {
		expect(entries.find((entry) => entry.key === 'receipt:r2')?.status).toBe(
			'unallocated',
		)
		expect(entries.find((entry) => entry.key === 'expense:e2')?.status).toBe(
			'late',
		)
	})

	it('summarizes receipts, expenses and the net balance', () => {
		const summary = summarizeLedger(entries)
		expect(summary.receipts).toBe(100000)
		expect(summary.expenses).toBe(36000)
		expect(summary.net).toBe(64000)
		expect(summary.unallocated).toBe(50000)
		expect(summary.pendingExpenses).toBe(12000)
	})

	it('filters by type, month, status and text', () => {
		expect(
			filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, type: 'receipt' }),
		).toHaveLength(2)
		expect(
			filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, month: '2026-10' }),
		).toHaveLength(2)
		expect(
			filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, status: 'late' }),
		).toHaveLength(1)
		expect(
			filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, query: 'bernard' }),
		).toHaveLength(3)
	})

	it('lists the months and a six-month trend ending today', () => {
		expect(ledgerMonths(entries)).toEqual(['2026-11', '2026-10'])
		const trend = monthlyTrend(entries, TODAY)
		expect(trend).toHaveLength(6)
		expect(trend.at(-1)?.month).toBe('2026-11')
		expect(trend.at(0)?.month).toBe('2026-06')
		expect(trend.at(-1)?.receipts).toBe(50000)
	})

	it('computes the VAT position', () => {
		expect(vatSummary(state, '2026-11')).toEqual({
			collected: 8333,
			deductible: 4000,
			due: 4333,
		})
	})
})

describe('search state', () => {
	it('parses and cleans', () => {
		expect(
			parseLedgerSearch({ type: 'recettes', mois: '2026-13', statut: 'late' }),
		).toEqual({ type: 'recettes', statut: 'late' })
		expect(cleanLedgerSearch({ mois: null, q: 'x' })).toEqual({ q: 'x' })
		expect(
			ledgerFiltersOf({ type: 'depenses', mois: '2026-10' }),
		).toMatchObject({
			type: 'expense',
			month: '2026-10',
		})
	})
})

describe('forms', () => {
	it('derives the VAT from the rate and the amount', () => {
		const withRate = applyVatRate(
			{ ...emptyExpenseValues(TODAY), amountHt: '100' },
			20,
		)
		expect(withRate.vatAmount).toBe('20')
		expect(changeAmountHt(withRate, '50').vatAmount).toBe('10')
	})

	it('lists expense errors', () => {
		expect(expenseFormErrors(emptyExpenseValues(TODAY))).toHaveLength(3)
		expect(
			expenseFormErrors({
				...emptyExpenseValues(TODAY),
				label: 'x',
				categoryId: 'c',
				amountHt: '12,5',
			}),
		).toEqual([])
	})

	it('checks a receipt against the invoice balance', () => {
		const values = {
			...emptyPaymentValues(TODAY),
			amount: '500',
			invoiceId: 'i1',
		}
		expect(paymentFormErrors(values, TODAY, () => 40000)).toHaveLength(1)
		expect(paymentFormErrors(values, TODAY, () => 60000)).toEqual([])
		expect(
			paymentFormErrors({ ...values, invoiceId: null }, TODAY, () => null),
		).toHaveLength(1)
	})
})
