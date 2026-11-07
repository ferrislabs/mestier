import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import {
	customersSummary,
	DEFAULT_FILTERS,
	filterCustomers,
	filtersOfSearch,
	hasActiveFilters,
	searchOfFilters,
	validateClientsSearch,
} from '#/pages/customers/customer-list-url-state'

function customer(overrides: Partial<Customer>): Customer {
	return {
		id: 'c1',
		organization_id: 'org',
		name: 'Boulangerie Lemoine',
		email: null,
		phone: null,
		status: 'CLIENT',
		kind: 'PROFESSIONAL',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

const JULIEN = '0190a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b'

describe('validateClientsSearch', () => {
	it('keeps only known statuses and owners', () => {
		expect(
			validateClientsSearch({ statut: 'prospect', responsable: JULIEN }),
		).toEqual({ statut: 'prospect', responsable: JULIEN })
		expect(
			validateClientsSearch({ statut: 'bogus', responsable: 'nobody', q: ' ' }),
		).toEqual({})
	})

	it('round-trips the filters', () => {
		const filters = { query: 'lem', status: 'client' as const, owner: 'none' }
		expect(filtersOfSearch(searchOfFilters(filters))).toEqual(filters)
		expect(searchOfFilters(DEFAULT_FILTERS)).toEqual({})
	})
})

describe('filterCustomers', () => {
	const list = [
		customer({ id: 'a', name: 'Éric Zola', status: 'PROSPECT' }),
		customer({ id: 'b', name: 'Agnès Brun', email: 'agnes@brun.fr' }),
		customer({ id: 'c', name: 'Archivé SA', status: 'ARCHIVED' }),
	]

	it('hides archived accounts by default and sorts by name', () => {
		expect(
			filterCustomers(list, DEFAULT_FILTERS).map((item) => item.id),
		).toEqual(['b', 'a'])
	})

	it('shows archived accounts only on demand', () => {
		expect(
			filterCustomers(list, {
				...DEFAULT_FILTERS,
				status: 'archived',
			}).map((item) => item.id),
		).toEqual(['c'])
	})

	it('matches accents-insensitively on name and email', () => {
		expect(
			filterCustomers(list, { ...DEFAULT_FILTERS, query: 'eric' }).map(
				(item) => item.id,
			),
		).toEqual(['a'])
		expect(
			filterCustomers(list, { ...DEFAULT_FILTERS, query: 'brun.fr' }).map(
				(item) => item.id,
			),
		).toEqual(['b'])
	})

	it('matches on the city, contacts and phones of the row', () => {
		const terms = (item: { id: string }) =>
			item.id === 'a'
				? { text: ['Rezé', 'Léa Duval'], phones: ['06 12 34 56 78'] }
				: { text: [], phones: [] }
		const ids = (query: string) =>
			filterCustomers(list, { ...DEFAULT_FILTERS, query }, terms).map(
				(item) => item.id,
			)
		expect(ids('reze')).toEqual(['a'])
		expect(ids('lea duval')).toEqual(['a'])
		expect(ids('0612345678')).toEqual(['a'])
		expect(ids('nantes')).toEqual([])
	})

	it('filters on the owner member of the customer', () => {
		const owned = [
			customer({ id: 'a', name: 'Éric Zola', owner_member_id: JULIEN }),
			customer({ id: 'b', name: 'Agnès Brun', owner_member_id: null }),
		]
		expect(
			filterCustomers(owned, { ...DEFAULT_FILTERS, owner: JULIEN }).map(
				(item) => item.id,
			),
		).toEqual(['a'])
		expect(
			filterCustomers(owned, { ...DEFAULT_FILTERS, owner: 'none' }).map(
				(item) => item.id,
			),
		).toEqual(['b'])
	})
})

describe('summaries', () => {
	it('counts clients and prospects with plurals', () => {
		expect(
			customersSummary([
				customer({ id: '1' }),
				customer({ id: '2' }),
				customer({ id: '3', status: 'PROSPECT' }),
			]),
		).toBe('2 clients, 1 prospect')
		expect(hasActiveFilters(DEFAULT_FILTERS)).toBe(false)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, query: 'x' })).toBe(true)
	})
})
