// MOCK: treasury ledger — no backend endpoint yet. Replace with organization expenses, receipts (payments not tied to an invoice) and expense categories endpoints.
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'
import { addDays, todayIso } from '#/pages/customers/lib/format'
import type {
	Expense,
	ExpenseCategory,
	LedgerState,
	OpenInvoice,
	PaymentMethod,
	Receipt,
} from '#/pages/treasury/types'

const CATEGORIES: readonly ExpenseCategory[] = [
	{ id: 'cat-materiaux', name: 'Matériaux', hue: 60 },
	{ id: 'cat-sous-traitance', name: 'Sous-traitance', hue: 200 },
	{ id: 'cat-vehicules', name: 'Véhicules et carburant', hue: 25 },
	{ id: 'cat-outillage', name: 'Outillage et consommables', hue: 110 },
	{ id: 'cat-assurances', name: 'Assurances', hue: 280 },
	{ id: 'cat-loyers', name: 'Loyers et charges', hue: 330 },
	{ id: 'cat-logiciels', name: 'Logiciels et télécoms', hue: 230 },
	{ id: 'cat-deplacements', name: 'Déplacements et repas', hue: 160 },
	{ id: 'cat-autres', name: 'Autres', hue: 0 },
]

const OPEN_INVOICES: readonly OpenInvoice[] = [
	{
		id: 'inv-112',
		label: 'F-2026-112',
		customerName: 'SCI Les Tilleuls',
		balance: 399190,
		vatRate: 20,
	},
	{
		id: 'inv-115',
		label: 'F-2026-115',
		customerName: 'Atelier Bois & Co',
		balance: 49000,
		vatRate: 10,
	},
	{
		id: 'inv-117',
		label: 'F-2026-117',
		customerName: 'Boulangerie Lemoine',
		balance: 905760,
		vatRate: 20,
	},
]

function ago(days: number): string {
	return addDays(todayIso(), -days)
}

function vat(amountHt: number, rate: number): number {
	return Math.round((amountHt * rate) / 100)
}

function receipt(
	id: string,
	days: number,
	amount: number,
	method: PaymentMethod,
	customerName: string,
	invoiceLabel: string | null,
	reference: string,
	source: Receipt['source'] = 'manual',
): Receipt {
	return {
		id,
		date: ago(days),
		amount,
		vatAmount: Math.round((amount * 20) / 120),
		method,
		source,
		customerId: null,
		customerName,
		invoiceLabel,
		reference,
		note: '',
	}
}

function expense(
	id: string,
	days: number,
	label: string,
	supplierName: string,
	categoryId: string,
	amountHt: number,
	options: Partial<Expense> = {},
): Expense {
	return {
		id,
		date: ago(days),
		label,
		supplierId: null,
		supplierName,
		categoryId,
		reference: '',
		amountHt,
		vatAmount: vat(amountHt, 20),
		dueDate: null,
		paid: true,
		paidAt: ago(days),
		method: 'transfer',
		advancedById: null,
		reimbursedAt: null,
		customerId: null,
		customerName: '',
		proofName: null,
		note: '',
		...options,
	}
}

const INITIAL: LedgerState = {
	categories: CATEGORIES,
	openInvoices: OPEN_INVOICES,
	receipts: [
		receipt(
			'r1',
			3,
			125000,
			'transfer',
			'Boulangerie Lemoine',
			'F-2026-109',
			'VIR 1105 LEMOINE',
		),
		receipt(
			'r2',
			12,
			49000,
			'card',
			'Atelier Bois & Co',
			null,
			'pi_3Qx92',
			'stripe',
		),
		receipt(
			'r3',
			35,
			310000,
			'transfer',
			'SCI Les Tilleuls',
			'F-2026-101',
			'VIR 0928 TILLEULS',
		),
		receipt(
			'r4',
			62,
			248000,
			'check',
			'M. et Mme Fabre',
			'F-2026-094',
			'CHQ 0042871',
		),
		receipt(
			'r5',
			95,
			520000,
			'transfer',
			'Boulangerie Lemoine',
			'F-2026-088',
			'VIR 0610 LEMOINE',
		),
		receipt(
			'r6',
			130,
			186000,
			'card',
			'Famille Durand',
			'F-2026-079',
			'pi_3Op11',
			'stripe',
		),
	],
	expenses: [
		expense(
			'e1',
			5,
			'Quincaillerie et visserie',
			'Quincaillerie Bernard',
			'cat-materiaux',
			84200,
			{
				paid: false,
				paidAt: null,
				method: null,
				dueDate: ago(-20),
				reference: 'FA-88213',
			},
		),
		expense(
			'e2',
			18,
			'Carburant fourgon',
			'Station Avia',
			'cat-vehicules',
			9600,
		),
		expense(
			'e3',
			40,
			'Assurance décennale, trimestre',
			'AXA France',
			'cat-assurances',
			64000,
		),
		expense(
			'e4',
			8,
			'Sous-traitance électricité',
			'Élec Pro 44',
			'cat-sous-traitance',
			120000,
			{
				paid: false,
				paidAt: null,
				method: null,
				dueDate: ago(3),
				reference: 'F-0231',
			},
		),
		expense('e5', 25, 'Petit matériel de chantier', '', 'cat-outillage', 5400, {
			advancedById: 'julien',
			paid: true,
			paidAt: ago(25),
			method: 'card',
		}),
		expense(
			'e6',
			70,
			'Loyer de l’atelier',
			'SCI Duval Immo',
			'cat-loyers',
			90000,
		),
		expense(
			'e7',
			105,
			'Panneaux de bois massif',
			'Bois du Maine',
			'cat-materiaux',
			231000,
		),
		expense('e8', 150, 'Abonnement logiciel', 'Mestier', 'cat-logiciels', 4900),
	],
}

export const ledgerStore = createMockStore<LedgerState>(INITIAL)

export function useLedger() {
	return { data: ledgerStore.use(), isLoading: false }
}

export function useSaveExpense() {
	return {
		isPending: false,
		mutateAsync: async (id: string | null, draft: Omit<Expense, 'id'>) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				expenses:
					id === null
						? [...state.expenses, { ...draft, id: `e-${crypto.randomUUID()}` }]
						: state.expenses.map((item) =>
								item.id === id ? { ...item, ...draft, id } : item,
							),
			}))
		},
	}
}

export function useDeleteExpense() {
	return {
		mutateAsync: async (id: string) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				expenses: state.expenses.filter((item) => item.id !== id),
			}))
		},
	}
}

export function useMarkExpensePaid() {
	return {
		mutateAsync: async (id: string, paidAt: string, method: PaymentMethod) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				expenses: state.expenses.map((item) =>
					item.id === id ? { ...item, paid: true, paidAt, method } : item,
				),
			}))
		},
	}
}

export function useReimburseExpense() {
	return {
		mutateAsync: async (id: string, reimbursedAt: string) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				expenses: state.expenses.map((item) =>
					item.id === id ? { ...item, reimbursedAt } : item,
				),
			}))
		},
	}
}

export function useRecordReceipt() {
	return {
		isPending: false,
		mutateAsync: async (draft: Omit<Receipt, 'id' | 'source'>) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				receipts: [
					...state.receipts,
					{ ...draft, id: `r-${crypto.randomUUID()}`, source: 'manual' },
				],
			}))
		},
	}
}

export function useAllocateReceipt() {
	return {
		mutateAsync: async (receiptId: string, invoice: OpenInvoice) => {
			await resolveAfterDelay(null)
			ledgerStore.set((state) => ({
				...state,
				receipts: state.receipts.map((item) =>
					item.id === receiptId
						? {
								...item,
								invoiceLabel: invoice.label,
								customerName: item.customerName || invoice.customerName,
							}
						: item,
				),
			}))
		},
	}
}
