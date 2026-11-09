import { describe, expect, it } from 'vitest'
import {
	EMPTY_FILTERS,
	filtersFromInputs,
	hasFilters,
	listSearchParams,
} from '#/pages/photo-library/lib/filters'

describe('listSearchParams', () => {
	it('sends only the page without filters', () => {
		expect(listSearchParams(EMPTY_FILTERS, 1, 48).toString()).toBe(
			'page=1&per_page=48',
		)
	})

	it('repeats tag and sets the other filters', () => {
		const params = listSearchParams(
			{ q: ' buis ', tags: ['a', 'b'], projectId: 'p1', toSort: true },
			2,
			48,
		)
		expect(params.getAll('tag')).toEqual(['a', 'b'])
		expect(params.get('q')).toBe('buis')
		expect(params.get('project_id')).toBe('p1')
		expect(params.get('to_sort')).toBe('true')
		expect(params.get('page')).toBe('2')
	})
})

describe('filtersFromInputs', () => {
	it('normalises the typed tags', () => {
		expect(
			filtersFromInputs({
				q: ' x ',
				tagInput: 'Buis, ALLÉE',
				projectId: null,
				toSort: false,
			}),
		).toEqual({
			q: 'x',
			tags: ['buis', 'allée'],
			projectId: null,
			toSort: false,
		})
	})
})

describe('hasFilters', () => {
	it('is false only for the empty filters', () => {
		expect(hasFilters(EMPTY_FILTERS)).toBe(false)
		expect(hasFilters({ ...EMPTY_FILTERS, toSort: true })).toBe(true)
	})
})
