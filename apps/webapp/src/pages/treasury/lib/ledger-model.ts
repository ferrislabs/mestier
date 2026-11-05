import {
	addDays,
	centsToInput,
	monthOf,
	parseAmount,
} from '#/pages/customers/lib/format'
import type {
	Expense,
	ExpenseDisplayState,
	LedgerEntry,
	LedgerKind,
	LedgerState,
	LedgerStatus,
	PaymentMethod,
	Receipt,
} from '#/pages/treasury/types'

export type LedgerTypeFilter = 'all' | LedgerKind

export type LedgerFilters = {
	type: LedgerTypeFilter
	month: string | null
	query: string
	categoryId: string | null
	status: LedgerStatus | null
}

export const DEFAULT_LEDGER_FILTERS: LedgerFilters = {
	type: 'all',
	month: null,
	query: '',
	categoryId: null,
	status: null,
}

export function expenseTotal(
	expense: Pick<Expense, 'amountHt' | 'vatAmount'>,
): number {
	return expense.amountHt + expense.vatAmount
}

export function expenseState(
	expense: Expense,
	today: string,
): ExpenseDisplayState {
	if (expense.advancedById !== null)
		return expense.reimbursedAt === null ? 'to_reimburse' : 'reimbursed'
	if (expense.paid) return 'paid'
	return expense.dueDate !== null && expense.dueDate < today
		? 'overdue'
		: 'to_pay'
}

export function canMarkPaid(expense: Expense): boolean {
	return expense.advancedById === null && !expense.paid
}

export function canReimburse(expense: Expense): boolean {
	return expense.advancedById !== null && expense.reimbursedAt === null
}

export function validatePaidDate(
	expense: Expense,
	paidAt: string,
	today: string,
): string[] {
	if (paidAt === '') return ['Renseigne la date de paiement.']
	if (paidAt > today) return ['La date de paiement ne peut pas être future.']
	if (paidAt < expense.date)
		return ['La date de paiement précède la date de la dépense.']
	return []
}

function receiptEntry(receipt: Receipt): LedgerEntry {
	return {
		key: `receipt:${receipt.id}`,
		kind: 'receipt',
		sourceId: receipt.id,
		date: receipt.date,
		label: receipt.invoiceLabel
			? `${receipt.amount < 0 ? 'Remboursement' : 'Paiement'} ${receipt.invoiceLabel}`
			: receipt.note || 'Recette non affectée',
		party: receipt.customerName,
		reference: receipt.reference,
		amount: receipt.amount,
		status:
			receipt.invoiceLabel === null && receipt.amount > 0
				? 'unallocated'
				: 'done',
		method: receipt.method,
		origin: receipt.source,
		categoryId: null,
		invoiceLabel: receipt.invoiceLabel,
		customerId: receipt.customerId,
	}
}

function expenseEntry(expense: Expense, today: string): LedgerEntry {
	const display = expenseState(expense, today)
	return {
		key: `expense:${expense.id}`,
		kind: 'expense',
		sourceId: expense.id,
		date: expense.date,
		label: expense.label,
		party: expense.supplierName,
		reference: expense.reference,
		amount: -expenseTotal(expense),
		status:
			display === 'overdue'
				? 'late'
				: display === 'to_pay'
					? 'pending'
					: display === 'to_reimburse'
						? 'to_reimburse'
						: 'done',
		method: expense.method,
		origin: 'manual',
		categoryId: expense.categoryId,
		invoiceLabel: null,
		customerId: expense.customerId,
	}
}

export function ledgerEntries(
	state: LedgerState,
	today: string,
): LedgerEntry[] {
	return [
		...state.receipts.map(receiptEntry),
		...state.expenses.map((expense) => expenseEntry(expense, today)),
	].sort((a, b) => b.date.localeCompare(a.date) || a.key.localeCompare(b.key))
}

export function filterLedger(
	entries: readonly LedgerEntry[],
	filters: LedgerFilters,
): LedgerEntry[] {
	const query = filters.query.trim().toLowerCase()
	return entries.filter((entry) => {
		if (filters.type !== 'all' && entry.kind !== filters.type) return false
		if (filters.month !== null && monthOf(entry.date) !== filters.month)
			return false
		if (filters.categoryId !== null && entry.categoryId !== filters.categoryId)
			return false
		if (filters.status !== null && entry.status !== filters.status) return false
		if (
			query !== '' &&
			!`${entry.label} ${entry.party} ${entry.reference}`
				.toLowerCase()
				.includes(query)
		)
			return false
		return true
	})
}

export type LedgerSummary = {
	receipts: number
	expenses: number
	net: number
	pendingExpenses: number
	unallocated: number
}

export function summarizeLedger(
	entries: readonly LedgerEntry[],
): LedgerSummary {
	let receipts = 0
	let expenses = 0
	let pendingExpenses = 0
	let unallocated = 0
	for (const entry of entries) {
		if (entry.kind === 'receipt') {
			receipts += entry.amount
			if (entry.status === 'unallocated') unallocated += entry.amount
		} else {
			expenses += -entry.amount
			if (entry.status === 'pending' || entry.status === 'late')
				pendingExpenses += -entry.amount
		}
	}
	return {
		receipts,
		expenses,
		net: receipts - expenses,
		pendingExpenses,
		unallocated,
	}
}

export function ledgerMonths(entries: readonly LedgerEntry[]): string[] {
	return [...new Set(entries.map((entry) => monthOf(entry.date)))]
		.sort()
		.reverse()
}

export type MonthTrend = { month: string; receipts: number; expenses: number }

export function monthlyTrend(
	entries: readonly LedgerEntry[],
	today: string,
	months = 6,
): MonthTrend[] {
	const keys: string[] = []
	let cursor = `${today.slice(0, 7)}-01`
	for (let index = 0; index < months; index += 1) {
		keys.unshift(monthOf(cursor))
		cursor = addDays(cursor, -1).slice(0, 7).concat('-01')
	}
	return keys.map((month) => {
		const inMonth = entries.filter((entry) => monthOf(entry.date) === month)
		const summary = summarizeLedger(inMonth)
		return { month, receipts: summary.receipts, expenses: summary.expenses }
	})
}

export type VatSummary = { collected: number; deductible: number; due: number }

export function vatSummary(
	state: LedgerState,
	month: string | null,
): VatSummary {
	const inMonth = (date: string) => month === null || monthOf(date) === month
	const collected = state.receipts
		.filter((receipt) => inMonth(receipt.date))
		.reduce((sum, receipt) => sum + receipt.vatAmount, 0)
	const deductible = state.expenses
		.filter((expense) => inMonth(expense.date))
		.reduce((sum, expense) => sum + expense.vatAmount, 0)
	return { collected, deductible, due: collected - deductible }
}

export function payables(state: LedgerState, today: string): Expense[] {
	return state.expenses.filter((expense) => {
		const display = expenseState(expense, today)
		return display === 'to_pay' || display === 'overdue'
	})
}

export function fitsBalance(balance: number, amount: number): boolean {
	return amount <= balance
}

export type LedgerSearchType = 'recettes' | 'depenses'

export type LedgerSearch = {
	type?: LedgerSearchType
	mois?: string
	q?: string
	categorie?: string
	statut?: LedgerStatus
}

export type LedgerSearchPatch = {
	[Key in keyof LedgerSearch]?: LedgerSearch[Key] | null
}

const MONTH = /^\d{4}-(0[1-9]|1[0-2])$/

const STATUSES: readonly LedgerStatus[] = [
	'done',
	'pending',
	'late',
	'unallocated',
	'to_reimburse',
]

function isStatus(value: unknown): value is LedgerStatus {
	return STATUSES.some((status) => status === value)
}

function textOf(value: unknown): string | null {
	if (typeof value === 'number') return String(value)
	if (typeof value !== 'string') return null
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function parseLedgerSearch(raw: Record<string, unknown>): LedgerSearch {
	const month = textOf(raw.mois)
	const query = textOf(raw.q)
	const category = textOf(raw.categorie)
	return {
		...(raw.type === 'recettes' || raw.type === 'depenses'
			? { type: raw.type }
			: {}),
		...(month !== null && MONTH.test(month) ? { mois: month } : {}),
		...(query !== null ? { q: query } : {}),
		...(category !== null ? { categorie: category } : {}),
		...(isStatus(raw.statut) ? { statut: raw.statut } : {}),
	}
}

export function cleanLedgerSearch(search: LedgerSearchPatch): LedgerSearch {
	return {
		...(search.type ? { type: search.type } : {}),
		...(search.mois ? { mois: search.mois } : {}),
		...(search.q ? { q: search.q } : {}),
		...(search.categorie ? { categorie: search.categorie } : {}),
		...(search.statut ? { statut: search.statut } : {}),
	}
}

export function updateLedgerSearch(
	current: LedgerSearch,
	patch: LedgerSearchPatch,
): LedgerSearch {
	return cleanLedgerSearch({ ...current, ...patch })
}

const TYPE_FILTERS: Record<LedgerSearchType, LedgerTypeFilter> = {
	recettes: 'receipt',
	depenses: 'expense',
}

export function ledgerFiltersOf(search: LedgerSearch): LedgerFilters {
	return {
		type: search.type ? TYPE_FILTERS[search.type] : 'all',
		month: search.mois ?? null,
		query: search.q ?? '',
		categoryId: search.categorie ?? null,
		status: search.statut ?? null,
	}
}

export function hasLedgerFilters(search: LedgerSearch): boolean {
	return Boolean(search.mois || search.q || search.categorie || search.statut)
}

export function ledgerTitle(type: LedgerSearchType | undefined): string {
	if (type === 'recettes') return 'Recettes'
	if (type === 'depenses') return 'Dépenses'
	return 'Journal'
}

export function ledgerDescription(type: LedgerSearchType | undefined): string {
	if (type === 'recettes')
		return 'Les paiements reçus, affectés ou non à une facture.'
	if (type === 'depenses')
		return 'Les achats et frais de l’entreprise, payés ou à payer.'
	return 'Toutes les recettes et dépenses, de la plus récente à la plus ancienne.'
}

export function ledgerStatusOptions(
	type: LedgerSearchType | undefined,
): LedgerStatus[] {
	if (type === 'recettes') return ['done', 'unallocated']
	if (type === 'depenses') return ['pending', 'late', 'to_reimburse', 'done']
	return [...STATUSES]
}

export type ExpenseFormValues = {
	date: string
	label: string
	supplierId: string | null
	categoryId: string
	categoryTouched: boolean
	reference: string
	amountHt: string
	vatAmount: string
	vatRate: number | null
	dueDate: string
	advancedById: string | null
	customerId: string | null
	proofName: string | null
	note: string
}

export function emptyExpenseValues(today: string): ExpenseFormValues {
	return {
		date: today,
		label: '',
		supplierId: null,
		categoryId: '',
		categoryTouched: false,
		reference: '',
		amountHt: '',
		vatAmount: '',
		vatRate: null,
		dueDate: '',
		advancedById: null,
		customerId: null,
		proofName: null,
		note: '',
	}
}

export function vatOf(base: number, rate: number): number {
	return Math.round((base * rate) / 100)
}

export function expenseValuesOf(expense: Expense): ExpenseFormValues {
	return {
		date: expense.date,
		label: expense.label,
		supplierId: expense.supplierId,
		categoryId: expense.categoryId,
		categoryTouched: true,
		reference: expense.reference,
		amountHt: centsToInput(expense.amountHt),
		vatAmount: centsToInput(expense.vatAmount),
		vatRate:
			[20, 10, 5.5, 0].find(
				(rate) =>
					expense.amountHt > 0 &&
					vatOf(expense.amountHt, rate) === expense.vatAmount,
			) ?? null,
		dueDate: expense.dueDate ?? '',
		advancedById: expense.advancedById,
		customerId: expense.customerId,
		proofName: expense.proofName,
		note: expense.note,
	}
}

export function applyVatRate(
	values: ExpenseFormValues,
	rate: number,
): ExpenseFormValues {
	const base = parseAmount(values.amountHt)
	return {
		...values,
		vatRate: rate,
		vatAmount:
			base === null ? values.vatAmount : centsToInput(vatOf(base, rate)),
	}
}

export function changeAmountHt(
	values: ExpenseFormValues,
	input: string,
): ExpenseFormValues {
	const base = parseAmount(input)
	return {
		...values,
		amountHt: input,
		vatAmount:
			values.vatRate !== null && base !== null
				? centsToInput(vatOf(base, values.vatRate))
				: values.vatAmount,
	}
}

export function changeVatAmount(
	values: ExpenseFormValues,
	input: string,
): ExpenseFormValues {
	return { ...values, vatAmount: input, vatRate: null }
}

function vatOfInput(input: string): number | null {
	return input.trim() === '' ? 0 : parseAmount(input)
}

export function expenseTotalOf(values: ExpenseFormValues): number | null {
	const base = parseAmount(values.amountHt)
	const vat = vatOfInput(values.vatAmount)
	return base === null || vat === null ? null : base + vat
}

export function expenseFormErrors(values: ExpenseFormValues): string[] {
	const errors: string[] = []
	if (values.label.trim() === '') errors.push('Le libellé est obligatoire.')
	if (values.date === '') errors.push('La date est obligatoire.')
	if (values.categoryId === '') errors.push('Choisis une catégorie.')
	const base = parseAmount(values.amountHt)
	if (base === null || base <= 0)
		errors.push('Le montant HT doit être un montant positif.')
	const vat = vatOfInput(values.vatAmount)
	if (vat === null || vat < 0)
		errors.push('La TVA n’est pas un montant valide.')
	return errors
}

export type PaymentFormValues = {
	date: string
	amount: string
	method: PaymentMethod
	customerId: string | null
	invoiceId: string | null
	reference: string
	note: string
}

export function emptyPaymentValues(today: string): PaymentFormValues {
	return {
		date: today,
		amount: '',
		method: 'transfer',
		customerId: null,
		invoiceId: null,
		reference: '',
		note: '',
	}
}

export function paymentFormErrors(
	values: PaymentFormValues,
	today: string,
	balanceOf: (invoiceId: string) => number | null,
): string[] {
	const errors: string[] = []
	const amount = parseAmount(values.amount)
	if (amount === null || amount === 0)
		errors.push('Le montant doit être différent de zéro.')
	if (values.date === '') errors.push('La date n’est pas valide.')
	else if (values.date > today)
		errors.push('La date ne peut pas être dans le futur.')
	if (values.invoiceId !== null) {
		const balance = balanceOf(values.invoiceId)
		if (balance !== null && amount !== null && amount > balance)
			errors.push('Le paiement dépasse le reste à payer de la facture.')
	} else if (values.customerId === null && values.note.trim() === '')
		errors.push('Précise le client ou une note pour cette recette.')
	return errors
}
