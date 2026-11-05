import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import { EMPTY_EXTRAS } from '#/pages/customers/feature/mocks/account-extras'
import {
	closedFilterFor,
	conversionNotice,
	DEFAULT_FILTERS,
	filterPipelineCustomers,
	filtersFromSearch,
	hasActiveFilters,
	parseSearch,
	searchFromFilters,
	visibleStages,
} from '#/pages/customers/lib/pipeline-filters'

function customer(overrides: Partial<Customer>): Customer {
	return {
		id: 'c1',
		organization_id: 'org',
		name: 'Mairie de Saint-Julien',
		email: null,
		phone: null,
		status: 'PROSPECT',
		pipeline_stage: 'NEW',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

describe('closed columns', () => {
	it('derives the filter from the two checkboxes', () => {
		expect(closedFilterFor(true, true)).toBe('tous')
		expect(closedFilterFor(true, false)).toBe('gagne')
		expect(closedFilterFor(false, true)).toBe('perdu')
		expect(closedFilterFor(false, false)).toBe('aucun')
	})

	it('hides the finished columns it is told to hide', () => {
		expect(visibleStages('tous')).toHaveLength(6)
		expect(visibleStages('aucun')).toEqual([
			'NEW',
			'CONTACTED',
			'QUALIFIED',
			'QUOTE_SENT',
		])
		expect(visibleStages('gagne')).toContain('WON')
		expect(visibleStages('gagne')).not.toContain('LOST')
	})
})

describe('search state', () => {
	it('drops unknown values and round-trips known ones', () => {
		expect(parseSearch({ termine: 'x', source: 'y' })).toEqual({})
		const filters = {
			query: 'lem',
			owner: 'julien',
			source: 'salon' as const,
			closed: 'perdu' as const,
		}
		expect(filtersFromSearch(searchFromFilters(filters))).toEqual(filters)
		expect(hasActiveFilters(DEFAULT_FILTERS)).toBe(false)
		expect(hasActiveFilters(filters)).toBe(true)
	})
})

describe('filterPipelineCustomers', () => {
	it('leaves archived accounts out and matches the query', () => {
		const rows = [
			customer({ id: 'a' }),
			customer({ id: 'b', status: 'ARCHIVED' }),
			customer({ id: 'c', name: 'Atelier Bois' }),
		]
		expect(
			filterPipelineCustomers(rows, () => EMPTY_EXTRAS, {
				...DEFAULT_FILTERS,
				query: 'atelier',
			}).map((item) => item.id),
		).toEqual(['c'])
		expect(
			filterPipelineCustomers(rows, () => EMPTY_EXTRAS, DEFAULT_FILTERS),
		).toHaveLength(2)
	})
})

describe('conversionNotice', () => {
	it('only announces a prospect becoming a client', () => {
		expect(conversionNotice(customer({}), 'WON')).toBe(
			'Mairie de Saint-Julien est maintenant client.',
		)
		expect(conversionNotice(customer({ status: 'CLIENT' }), 'WON')).toBeNull()
		expect(conversionNotice(customer({}), 'LOST')).toBeNull()
	})
})
