import { describe, expect, it } from 'vitest'
import {
	bpOfVatRate,
	type CatalogRow,
	catalogSummary,
	catalogUsage,
	DEFAULT_FILTERS,
	emptyForm,
	evaluateForm,
	filterCatalog,
	filtersOf,
	hasSameReference,
	marginOf,
	marginTone,
	searchOf,
	usageLabel,
	usageOf,
	validateCatalogSearch,
	vatRateOfBp,
} from '#/pages/catalog/catalog-model'
import {
	type CatalogExtras,
	EMPTY_CATALOG_EXTRAS,
} from '#/pages/catalog/feature/mocks/catalog-extras'

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

describe('catalog usage', () => {
	it('counts the quote and invoice lines that use each item', () => {
		const usage = catalogUsage({
			quoteLines: ['a', 'a', 'b', null],
			invoiceLines: ['a', null],
		})
		expect(usageOf(usage, 'a')).toEqual({ quotes: 2, invoices: 1, total: 3 })
		expect(usageOf(usage, 'b')).toEqual({ quotes: 1, invoices: 0, total: 1 })
	})

	it('reports an unknown item as never used', () => {
		expect(
			usageOf(catalogUsage({ quoteLines: [], invoiceLines: [] }), 'zzz'),
		).toEqual({ quotes: 0, invoices: 0, total: 0 })
	})

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
