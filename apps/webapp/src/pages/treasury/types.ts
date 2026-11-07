export type PaymentMethod =
	| 'card'
	| 'transfer'
	| 'check'
	| 'cash'
	| 'direct_debit'

export type PaymentSource = 'manual' | 'stripe'

export type ExpenseCategory = {
	id: string
	name: string
	hue: number
}

export type Expense = {
	id: string
	date: string
	label: string
	supplierId: string | null
	supplierName: string
	categoryId: string
	reference: string
	amountHt: number
	vatAmount: number
	dueDate: string | null
	paid: boolean
	paidAt: string | null
	method: PaymentMethod | null
	advancedById: string | null
	reimbursedAt: string | null
	customerId: string | null
	customerName: string
	proofName: string | null
	note: string
	total: number
	state: ExpenseDisplayState
	daysLate: number
}

export type Receipt = {
	id: string
	date: string
	amount: number
	method: PaymentMethod
	source: PaymentSource
	customerId: string | null
	customerName: string
	invoiceLabel: string | null
	reference: string
	note: string
}

export type OpenInvoice = {
	id: string
	label: string
	customerName: string
	balance: number
}

export type LedgerKind = 'receipt' | 'expense'

export type LedgerStatus =
	| 'done'
	| 'pending'
	| 'late'
	| 'unallocated'
	| 'to_reimburse'

export type ExpenseDisplayState =
	| 'to_pay'
	| 'overdue'
	| 'paid'
	| 'to_reimburse'
	| 'reimbursed'

export type LedgerEntry = {
	key: string
	kind: LedgerKind
	sourceId: string
	date: string
	label: string
	party: string
	reference: string
	amount: number
	status: LedgerStatus
	method: PaymentMethod | null
	origin: PaymentSource
	categoryId: string | null
	invoiceLabel: string | null
	customerId: string | null
}

export const PAYMENT_METHOD_LABELS: Record<PaymentMethod, string> = {
	card: 'Carte bancaire',
	transfer: 'Virement',
	check: 'Chèque',
	cash: 'Espèces',
	direct_debit: 'Prélèvement',
}

export const PAYMENT_METHODS: readonly PaymentMethod[] = [
	'transfer',
	'card',
	'check',
	'cash',
	'direct_debit',
]

export const LEDGER_STATUS_LABELS: Record<LedgerStatus, string> = {
	done: 'Réglé',
	pending: 'À payer',
	late: 'En retard',
	unallocated: 'À affecter',
	to_reimburse: 'À rembourser',
}

export const EXPENSE_STATE_LABELS: Record<ExpenseDisplayState, string> = {
	to_pay: 'À payer',
	overdue: 'En retard',
	paid: 'Payée',
	to_reimburse: 'À rembourser',
	reimbursed: 'Remboursée',
}
