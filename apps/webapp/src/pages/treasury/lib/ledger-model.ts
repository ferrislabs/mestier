import {
	centsToInput,
	monthOf,
	parseAmount,
} from '#/pages/customers/lib/format'
import type {
	Expense,
	ExpenseDisplayState,
	LedgerEntry,
	LedgerKind,
	LedgerStatus,
	PaymentMethod,
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

export function expenseTotal(expense: Pick<Expense, 'total'>): number {
	return expense.total
}

export function expenseState(
	expense: Pick<Expense, 'state'>,
): ExpenseDisplayState {
	return expense.state
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

export type MonthTrend = { month: string; receipts: number; expenses: number }

export type VatSummary = { collected: number; deductible: number; due: number }

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
