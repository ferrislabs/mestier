import { describe, expect, it } from 'vitest'
import {
	filtersFromInputs,
	hasFilters,
	listSearchParams,
} from '#/pages/knowledge/lib/filters'

describe('knowledge filters', () => {
	it('trims the search and splits the tags', () => {
		expect(
			filtersFromInputs({ q: '  gazon ', tagInput: 'Devis, 2026' }),
		).toEqual({ q: 'gazon', tags: ['devis', '2026'] })
	})

	it('has no filter when both inputs are blank', () => {
		expect(hasFilters(filtersFromInputs({ q: ' ', tagInput: ' , ' }))).toBe(
			false,
		)
		expect(hasFilters({ q: '', tags: ['cgv'] })).toBe(true)
	})

	it('sends one tag parameter per tag and no empty search', () => {
		const params = listSearchParams({ q: '', tags: ['a', 'b'] }, 2, 10)
		expect(params.get('page')).toBe('2')
		expect(params.get('per_page')).toBe('10')
		expect(params.has('q')).toBe(false)
		expect(params.getAll('tag')).toEqual(['a', 'b'])
		expect(listSearchParams({ q: ' paiement ', tags: [] }, 1).get('q')).toBe(
			'paiement',
		)
	})
})
