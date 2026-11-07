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
	ledgerFiltersOf,
	parseLedgerSearch,
	paymentFormErrors,
	validatePaidDate,
} from '#/pages/treasury/lib/ledger-model'
import type { Expense, LedgerEntry } from '#/pages/treasury/types'

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
		total: 12000,
		state: 'paid',
		daysLate: 0,
		...overrides,
	}
}

function entry(overrides: Partial<LedgerEntry>): LedgerEntry {
	return {
		key: 'receipt:r1',
		kind: 'receipt',
		sourceId: 'r1',
		date: '2026-11-02',
		label: 'Paiement F-2026-109',
		party: 'Lemoine',
		reference: '',
		amount: 50000,
		status: 'done',
		method: 'transfer',
		origin: 'manual',
		categoryId: null,
		invoiceLabel: 'F-2026-109',
		customerId: null,
		...overrides,
	}
}

describe('expense helpers', () => {
	it('reads the state and the total computed by the API', () => {
		expect(expenseState(expense({ state: 'overdue' }))).toBe('overdue')
		expect(expenseTotal(expense({ total: 12000 }))).toBe(12000)
	})

	it('offers the right action', () => {
		expect(canMarkPaid(expense({ paid: false }))).toBe(true)
		expect(canMarkPaid(expense({ advancedById: 'a' }))).toBe(false)
		expect(canReimburse(expense({ advancedById: 'a' }))).toBe(true)
		expect(
			canReimburse(expense({ advancedById: 'a', reimbursedAt: TODAY })),
		).toBe(false)
	})

	it('validates a payment date', () => {
		const e = expense({ date: '2026-11-01' })
		expect(validatePaidDate(e, '2026-11-03', TODAY)).toEqual([])
		expect(validatePaidDate(e, '2026-12-01', TODAY)).toHaveLength(1)
		expect(validatePaidDate(e, '2026-10-01', TODAY)).toHaveLength(1)
		expect(validatePaidDate(e, '', TODAY)).toHaveLength(1)
	})
})

describe('filterLedger', () => {
	const entries = [
		entry({ key: 'receipt:r1' }),
		entry({
			key: 'receipt:r2',
			date: '2026-10-05',
			status: 'unallocated',
			invoiceLabel: null,
		}),
		entry({
			key: 'expense:e1',
			kind: 'expense',
			label: 'Quincaillerie',
			party: 'Bernard',
			amount: -12000,
			categoryId: 'cat-materiaux',
		}),
		entry({
			key: 'expense:e2',
			kind: 'expense',
			date: '2026-10-10',
			party: 'Bernard',
			status: 'late',
			amount: -12000,
		}),
	]

	it('filters by type, month, status, category and text', () => {
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
			filterLedger(entries, {
				...DEFAULT_LEDGER_FILTERS,
				categoryId: 'cat-materiaux',
			}),
		).toHaveLength(1)
		expect(
			filterLedger(entries, { ...DEFAULT_LEDGER_FILTERS, query: 'bernard' }),
		).toHaveLength(2)
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
