import { describe, expect, it } from 'vitest'
import type { Supplier } from '#/hooks/use-suppliers'
import { EMPTY_SUPPLIER_EXTRAS } from '#/pages/suppliers/feature/mocks/supplier-extras'
import { emptyContact } from '#/pages/suppliers/lib/supplier-form'
import {
	formatLegalId,
	supplierRows,
	supplierTotals,
} from '#/pages/suppliers/lib/supplier-model'
import type { SupplierExtras } from '#/pages/suppliers/types'
import type { Expense, ExpenseCategory } from '#/pages/treasury/types'

const TODAY = '2026-11-05'

function supplier(id: string, name: string, patch: Partial<Supplier> = {}) {
	return {
		id,
		organization_id: 'org',
		name,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...patch,
	} as Supplier
}

function expense(patch: Partial<Expense>): Expense {
	return {
		id: 'x',
		date: '2026-10-01',
		label: 'Achat',
		supplierId: 'pointp',
		supplierName: '',
		categoryId: 'cat-materiaux',
		reference: '',
		amountHt: 1000,
		vatAmount: 200,
		dueDate: null,
		paid: false,
		paidAt: null,
		method: null,
		advancedById: null,
		reimbursedAt: null,
		customerId: null,
		customerName: '',
		proofName: null,
		note: '',
		...patch,
	}
}

const CATEGORIES: ExpenseCategory[] = [
	{ id: 'cat-materiaux', name: 'Matériaux', hue: 60 },
	{ id: 'cat-assurances', name: 'Assurances', hue: 280 },
]

const SUPPLIERS = [
	supplier('pointp', 'Point.P', {
		registration_number: '30112345600021',
		email: 'contact@pointp.fr',
	}),
	supplier('axa', 'AXA Assurances'),
	supplier('wurth', 'Würth'),
]

const EXTRAS: Record<string, SupplierExtras> = {
	pointp: {
		...EMPTY_SUPPLIER_EXTRAS,
		defaultCategoryId: 'cat-materiaux',
		contacts: [
			{ ...emptyContact('c'), firstName: 'Gaëtan', lastName: 'Rivière' },
		],
	},
	axa: { ...EMPTY_SUPPLIER_EXTRAS, defaultCategoryId: 'cat-assurances' },
	wurth: { ...EMPTY_SUPPLIER_EXTRAS, archived: true },
}

const EXPENSES: Expense[] = [
	expense({
		id: 'a',
		amountHt: 61250,
		vatAmount: 12250,
		dueDate: '2026-10-20',
	}),
	expense({ id: 'b', date: '2026-09-01', paid: true, paidAt: '2026-09-10' }),
	expense({ id: 'c', date: '2024-01-01', paid: true, paidAt: '2024-01-02' }),
	expense({ id: 'd', supplierId: 'axa', amountHt: 5000, vatAmount: 0 }),
]

function rows(filters: { archived?: boolean; query?: string } = {}) {
	return supplierRows({
		suppliers: SUPPLIERS,
		extras: EXTRAS,
		expenses: EXPENSES,
		categories: CATEGORIES,
		today: TODAY,
		filters,
	})
}

function find(id: string, list = rows()) {
	const found = list.find((row) => row.supplier.id === id)
	if (!found) throw new Error(`missing ${id}`)
	return found
}

describe('supplierRows', () => {
	it('lists the active suppliers by name, accents folded', () => {
		expect(rows().map((row) => row.supplier.id)).toEqual(['axa', 'pointp'])
	})

	it('lists the archived suppliers apart', () => {
		expect(rows({ archived: true }).map((row) => row.supplier.id)).toEqual([
			'wurth',
		])
	})

	it('sums the last twelve months and what is left to pay', () => {
		const row = find('pointp')
		expect(row.spend12m).toBe(73500 + 1200)
		expect(row.toPay).toBe(73500)
		expect(row.overdue).toBe(true)
		const axa = find('axa')
		expect(axa.spend12m).toBe(5000)
		expect(axa.toPay).toBe(5000)
		expect(axa.overdue).toBe(false)
	})

	it('lets old expenses fall out of the twelve-month window', () => {
		expect(find('pointp', rows()).spend12m).toBe(74700)
		const later = supplierRows({
			suppliers: SUPPLIERS,
			extras: EXTRAS,
			expenses: EXPENSES,
			categories: CATEGORIES,
			today: '2028-01-01',
			filters: {},
		})
		expect(find('pointp', later).spend12m).toBe(0)
		expect(find('pointp', later).toPay).toBe(73500)
	})

	it('carries the main contact and the default category', () => {
		const row = find('pointp')
		expect(row.contact).toBe('Gaëtan Rivière')
		expect(row.category?.name).toBe('Matériaux')
		expect(find('axa').contact).toBe('')
	})

	it('falls back on the API e-mail when no contact is stored', () => {
		expect(find('pointp').contactEmail).toBe('contact@pointp.fr')
	})

	it('searches name, SIRET, contact and category without accents', () => {
		const ids = (query: string) => rows({ query }).map((row) => row.supplier.id)
		expect(ids('point.p')).toEqual(['pointp'])
		expect(ids('riviere')).toEqual(['pointp'])
		expect(ids('assurances')).toEqual(['axa'])
		expect(ids('30112345600021')).toEqual(['pointp'])
		expect(ids('introuvable')).toEqual([])
	})
})

describe('supplierTotals', () => {
	it('adds the rows up', () => {
		expect(supplierTotals(rows())).toEqual({
			spend12m: 74700 + 5000,
			toPay: 73500 + 5000,
			overdue: true,
		})
	})
})

describe('formatLegalId', () => {
	it('groups SIRET and SIREN digits', () => {
		expect(formatLegalId('30112345600021')).toBe('301 123 456 00021')
		expect(formatLegalId('301123456')).toBe('301 123 456')
		expect(formatLegalId('')).toBe('')
		expect(formatLegalId('abc')).toBe('abc')
	})
})
