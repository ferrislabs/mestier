import { describe, expect, it } from 'vitest'
import type { Product, ServiceRate } from '#/hooks/use-reference-catalog'
import {
	bpOfVatRate,
	type CatalogExtras,
	type CatalogRow,
	catalogRows,
	catalogSummary,
	DEFAULT_FILTERS,
	EMPTY_CATALOG_EXTRAS,
	emptyForm,
	evaluateForm,
	filterCatalog,
	filtersOf,
	hasSameReference,
	marginOf,
	marginTone,
	searchOf,
	usageLabel,
	validateCatalogSearch,
	vatRateOfBp,
} from '#/pages/catalog/catalog-model'

function row(overrides: Partial<CatalogRow>): CatalogRow {
	return {
		id: 'p1',
		kind: 'product',
		reference: 'FEN-1',
		name: 'Fenêtre',
		description: '',
		unit: 'UNIT',
		unitPriceCents: 52000,
		vatRate: 10,
		extras: EMPTY_CATALOG_EXTRAS,
		usage: { quotes: 0, invoices: 0, total: 0 },
		product: null,
		serviceRate: null,
		...overrides,
	}
}

describe('vat conversion', () => {
	it('maps basis points to rates and back', () => {
		expect(vatRateOfBp(2000)).toBe(20)
		expect(vatRateOfBp(550)).toBe(5.5)
		expect(vatRateOfBp(1234)).toBeNull()
		expect(vatRateOfBp(undefined)).toBeNull()
		expect(bpOfVatRate(5.5)).toBe(550)
		expect(bpOfVatRate(null)).toBeNull()
	})
})

describe('margin', () => {
	it('needs a cost and a positive price', () => {
		expect(marginOf(52000, null)).toBeNull()
		expect(marginOf(0, 100)).toBeNull()
		expect(marginOf(52000, 31000)).toEqual({ amount: 21000, percent: 40.4 })
	})

	it('grades the percentage', () => {
		expect(marginTone(40)).toBe('ok')
		expect(marginTone(25)).toBe('warn')
		expect(marginTone(5)).toBe('alert')
	})
})

describe('filterCatalog', () => {
	const rows = [
		row({ id: 'a', name: 'Béton', reference: 'B-1', kind: 'product' }),
		row({ id: 'b', name: 'Pose', reference: '', kind: 'service' }),
		row({ id: 'c', name: 'Ancien', reference: 'OLD', kind: 'product' }),
	]
	const extrasOf = (id: string): CatalogExtras => ({
		...EMPTY_CATALOG_EXTRAS,
		category: id === 'a' ? 'Gros œuvre' : '',
		archived: id === 'c',
	})

	it('hides archived items unless asked', () => {
		expect(
			filterCatalog(rows, extrasOf, DEFAULT_FILTERS).map((item) => item.id),
		).toEqual(['b', 'a'])
		expect(
			filterCatalog(rows, extrasOf, { ...DEFAULT_FILTERS, archived: true }).map(
				(item) => item.id,
			),
		).toEqual(['b', 'a', 'c'])
	})

	it('filters by kind, category and query', () => {
		expect(
			filterCatalog(rows, extrasOf, {
				...DEFAULT_FILTERS,
				type: 'service',
			}).map((item) => item.id),
		).toEqual(['b'])
		expect(
			filterCatalog(rows, extrasOf, {
				...DEFAULT_FILTERS,
				category: 'Gros œuvre',
			}).map((item) => item.id),
		).toEqual(['a'])
		expect(
			filterCatalog(rows, extrasOf, { ...DEFAULT_FILTERS, query: 'beton' }).map(
				(item) => item.id,
			),
		).toEqual(['a'])
	})

	it('summarizes active and archived items', () => {
		expect(catalogSummary([], extrasOf)).toBe('Aucun article')
		expect(catalogSummary(rows, extrasOf)).toBe(
			'2 articles actifs (1 service, 1 produit), 1 archivé',
		)
	})
})

describe('search state', () => {
	it('round-trips', () => {
		const filters = {
			query: 'a',
			type: 'service' as const,
			category: 'X',
			archived: true,
		}
		expect(filtersOf(searchOf(filters))).toEqual(filters)
		expect(validateCatalogSearch({ type: 'nope', archives: 'true' })).toEqual({
			archives: true,
		})
	})
})

describe('evaluateForm', () => {
	it('flags the required fields', () => {
		const evaluation = evaluateForm(emptyForm(20))
		expect(evaluation.valid).toBe(false)
		expect(evaluation.errors.name).toBeDefined()
		expect(evaluation.errors.price).toBe('Le prix HT est obligatoire.')
	})

	it('accepts a priced item with an optional cost', () => {
		const evaluation = evaluateForm({
			...emptyForm(20),
			name: 'Pose',
			price: '52,50',
			cost: '30',
		})
		expect(evaluation.valid).toBe(true)
		expect(evaluation.priceCents).toBe(5250)
		expect(evaluation.costCents).toBe(3000)
	})

	it('refuses a malformed cost', () => {
		expect(
			evaluateForm({ ...emptyForm(20), name: 'x', price: '1', cost: 'abc' })
				.errors.cost,
		).toBeDefined()
	})
})

describe('hasSameReference', () => {
	it('ignores the edited row and blank references', () => {
		const rows = [row({ id: 'a', reference: 'FEN-1' })]
		expect(hasSameReference(rows, ' fen-1 ', null)).toBe(true)
		expect(hasSameReference(rows, 'fen-1', 'a')).toBe(false)
		expect(hasSameReference(rows, '', null)).toBe(false)
	})
})

const API_FIELDS = {
	organization_id: 'o1',
	created_at: '2026-11-01T00:00:00Z',
	updated_at: '2026-11-01T00:00:00Z',
	category: 'Gros œuvre',
	billing_frequency: 'MONTHLY',
	cost_cents: 31000,
	archived_at: null,
	usage_count: 3,
	quote_usage_count: 2,
	invoice_usage_count: 1,
} as const

describe('catalogRows', () => {
	it('carries the catalogue fields and the usage computed by the API', () => {
		const product: Product = {
			...API_FIELDS,
			id: 'p1',
			name: 'Béton',
			sku: 'B-1',
			unit: 'M3',
			unit_price_cents: 52000,
			default_vat_rate_bp: 2000,
			description: null,
			photo_keys: [],
			datasheet_keys: [],
		}
		const serviceRate: ServiceRate = {
			...API_FIELDS,
			id: 's1',
			label: 'Pose',
			unit: 'HOUR',
			rate_cents: 5500,
			default_vat_rate_bp: null,
			description: null,
			archived_at: '2026-11-02T00:00:00Z',
			cost_cents: null,
			billing_frequency: 'ONE_SHOT',
			usage_count: 0,
			quote_usage_count: 0,
			invoice_usage_count: 0,
		}

		const [first, second] = catalogRows([product], [serviceRate])

		expect(first?.extras).toEqual({
			category: 'Gros œuvre',
			cost: 31000,
			frequency: 'monthly',
			archived: false,
		})
		expect(first?.usage).toEqual({ quotes: 2, invoices: 1, total: 3 })
		expect(second?.extras).toEqual({
			category: 'Gros œuvre',
			cost: null,
			frequency: 'one_shot',
			archived: true,
		})
		expect(second?.usage).toEqual({ quotes: 0, invoices: 0, total: 0 })
	})
})

describe('usageLabel', () => {
	it('describes the usage in French', () => {
		expect(usageLabel({ quotes: 0, invoices: 0, total: 0 })).toEqual({
			main: 'Jamais utilisé',
			detail: null,
		})
		expect(usageLabel({ quotes: 1, invoices: 0, total: 1 })).toEqual({
			main: '1 ligne',
			detail: '1 en devis, 0 en factures',
		})
		expect(usageLabel({ quotes: 2, invoices: 1, total: 3 }).main).toBe(
			'3 lignes',
		)
	})
})
