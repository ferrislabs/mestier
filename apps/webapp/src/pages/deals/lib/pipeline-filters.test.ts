import { describe, expect, it } from 'vitest'
import {
	accountOptions,
	closedFilterFor,
	conversionNotice,
	DEFAULT_FILTERS,
	dealQuotes,
	filterDeals,
	filtersFromSearch,
	hasActiveFilters,
	isOverdue,
	latestQuote,
	parseSearch,
	reopenStage,
	searchFromFilters,
	visibleStages,
} from '#/pages/deals/lib/pipeline-filters'
import { customer, deal, quote, TODAY } from '#/pages/deals/lib/testing'

describe('search params', () => {
	it('validates the search and drops defaults', () => {
		expect(parseSearch({})).toEqual({})
		expect(
			parseSearch({
				q: 'fenêtres',
				responsable: 'julien',
				source: 'salon',
				termine: 'gagne',
			}),
		).toEqual({
			q: 'fenêtres',
			responsable: 'julien',
			source: 'salon',
			termine: 'gagne',
		})
		expect(
			parseSearch({
				q: '',
				responsable: 'inconnu',
				source: 'telepathie',
				termine: 'tous',
			}),
		).toEqual({})
		expect(parseSearch({ termine: 'aucun' })).toEqual({ termine: 'aucun' })
		expect(parseSearch({ q: 12, termine: 3 })).toEqual({})
	})

	it('round-trips filters and search params', () => {
		expect(filtersFromSearch({})).toEqual(DEFAULT_FILTERS)
		const filters = {
			query: 'dupont',
			owner: 'karim',
			source: 'salon',
			closed: 'perdu',
		} as const
		expect(filtersFromSearch(searchFromFilters(filters))).toEqual(filters)
		expect(searchFromFilters(DEFAULT_FILTERS)).toEqual({})
	})
})

describe('filters', () => {
	it('detects active filters', () => {
		expect(hasActiveFilters(DEFAULT_FILTERS)).toBe(false)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, query: '  ' })).toBe(false)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, closed: 'aucun' })).toBe(true)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, owner: 'julien' })).toBe(true)
	})

	it('derives the closed columns from the two checkboxes', () => {
		expect(closedFilterFor(true, true)).toBe('tous')
		expect(closedFilterFor(true, false)).toBe('gagne')
		expect(closedFilterFor(false, true)).toBe('perdu')
		expect(closedFilterFor(false, false)).toBe('aucun')
		const open = ['new', 'visit', 'quote_sent', 'negotiation']
		expect(visibleStages('tous')).toEqual([...open, 'won', 'lost'])
		expect(visibleStages('gagne')).toEqual([...open, 'won'])
		expect(visibleStages('perdu')).toEqual([...open, 'lost'])
		expect(visibleStages('aucun')).toEqual(open)
	})
})

describe('filterDeals', () => {
	const customers = [
		customer({ id: 'c1', name: 'Famille Dupont' }),
		customer({ id: 'c2', name: 'SCI Les Tilleuls' }),
	]
	const deals = [
		deal({
			id: 'd-dupont',
			customerId: 'c1',
			title: 'Escalier Quart Tournant',
			ownerId: 'julien',
			source: 'site_web',
		}),
		deal({
			id: 'd-sci',
			customerId: 'c2',
			title: 'Rénovation cage d’escalier',
			ownerId: 'karim',
			source: 'salon',
		}),
		deal({ id: 'd-orphan', customerId: 'zz', title: 'Terrasse' }),
	]
	const run = (patch: Partial<typeof DEFAULT_FILTERS>) =>
		filterDeals(deals, customers, { ...DEFAULT_FILTERS, ...patch }).map(
			(item) => item.id,
		)

	it('searches by title or customer, ignoring case and accents', () => {
		expect(run({})).toHaveLength(3)
		expect(run({ query: 'ESCALIER' })).toEqual(['d-dupont', 'd-sci'])
		expect(run({ query: 'cage d’escalier' })).toEqual(['d-sci'])
		expect(run({ query: "cage d'escalier" })).toEqual(['d-sci'])
		expect(run({ query: 'famille dupont' })).toEqual(['d-dupont'])
		expect(run({ query: 'tilleuls' })).toEqual(['d-sci'])
		expect(run({ query: 'zzz' })).toEqual([])
	})

	it('filters by owner and source', () => {
		expect(run({ owner: 'julien' })).toEqual(['d-dupont'])
		expect(run({ source: 'salon' })).toEqual(['d-sci'])
		expect(run({ owner: 'julien', source: 'salon' })).toEqual([])
	})
})

describe('isOverdue', () => {
	const open = deal({ expectedCloseDate: '2026-11-04' })

	it('is true only for an open deal past its close date', () => {
		expect(isOverdue(open, TODAY)).toBe(true)
		expect(isOverdue({ ...open, expectedCloseDate: TODAY }, TODAY)).toBe(false)
		expect(isOverdue({ ...open, expectedCloseDate: null }, TODAY)).toBe(false)
		expect(isOverdue({ ...open, stage: 'won' }, TODAY)).toBe(false)
		expect(isOverdue({ ...open, stage: 'lost' }, TODAY)).toBe(false)
	})
})

describe('quotes of a deal', () => {
	const quotes = [
		quote({ id: 'q1', dealId: 'd1', createdAt: '2026-10-02' }),
		quote({ id: 'q2', dealId: 'd1', createdAt: '2026-10-09' }),
		quote({ id: 'q3', dealId: 'd2' }),
	]

	it('links quotes and finds the latest', () => {
		expect(dealQuotes(quotes, 'd1').map((item) => item.id)).toEqual([
			'q1',
			'q2',
		])
		expect(latestQuote(dealQuotes(quotes, 'd1'))?.id).toBe('q2')
		expect(latestQuote([])).toBeUndefined()
	})

	it('reopens at the first stage the quotes allow', () => {
		expect(reopenStage([])).toBe('new')
		expect(reopenStage([quote({ status: 'draft' })])).toBe('new')
		expect(reopenStage([quote({ status: 'sent' })])).toBe('quote_sent')
	})
})

describe('accountOptions', () => {
	it('hides archived customers except the current one, sorted by name', () => {
		const customers = [
			customer({ id: 'a', name: 'Zoé' }),
			customer({ id: 'b', name: 'Émile', status: 'ARCHIVED' }),
			customer({ id: 'c', name: 'Alice' }),
		]
		expect(accountOptions(customers, null).map((item) => item.id)).toEqual([
			'c',
			'a',
		])
		expect(accountOptions(customers, 'b').map((item) => item.id)).toEqual([
			'c',
			'b',
			'a',
		])
	})
})

describe('conversionNotice', () => {
	it('announces the conversion of a prospect that wins', () => {
		expect(conversionNotice(customer(), 'won')).toBe(
			'Famille Dupont est maintenant client.',
		)
	})

	it('stays silent for a client, a loss or a plain move', () => {
		expect(conversionNotice(customer({ status: 'CLIENT' }), 'won')).toBeNull()
		expect(conversionNotice(customer(), 'lost')).toBeNull()
		expect(conversionNotice(customer(), 'visit')).toBeNull()
		expect(conversionNotice(undefined, 'won')).toBeNull()
	})
})
