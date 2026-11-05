import { describe, expect, it } from 'vitest'
import {
	filtersOfSearch,
	searchOfFilters,
} from '#/pages/projects/lib/list-search'
import { DEFAULT_FILTERS } from '#/pages/projects/lib/project-list'

describe('list search', () => {
	it('reads the defaults from a bare search', () => {
		expect(filtersOfSearch({ archived: false })).toEqual(DEFAULT_FILTERS)
	})

	it('keeps the legacy archived flag as the all-projects filter', () => {
		expect(filtersOfSearch({ archived: true }).status).toBe('tous')
	})

	it('lets an explicit status win over the legacy flag', () => {
		expect(filtersOfSearch({ archived: true, statut: 'archives' }).status).toBe(
			'archives',
		)
	})

	it('writes only what differs from the defaults', () => {
		expect(searchOfFilters(DEFAULT_FILTERS, undefined)).toEqual({
			archived: false,
		})
		expect(
			searchOfFilters({ query: 'duval', status: 'tous', client: 'c1' }, 'p1'),
		).toEqual({
			archived: false,
			projectId: 'p1',
			statut: 'tous',
			q: 'duval',
			client: 'c1',
		})
	})
})
