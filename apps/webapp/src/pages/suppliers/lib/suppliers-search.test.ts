import { describe, expect, it } from 'vitest'
import type { Supplier } from '#/hooks/use-suppliers'
import {
	filterSuppliers,
	formatLegalId,
} from '#/pages/suppliers/lib/suppliers-search'

function supplier(overrides: Partial<Supplier>): Supplier {
	return {
		id: 's1',
		organization_id: 'org',
		name: 'Pépinières Martin',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	} as Supplier
}

describe('formatLegalId', () => {
	it('groups SIRET and SIREN digits', () => {
		expect(formatLegalId('82345678900019')).toBe('823 456 789 00019')
		expect(formatLegalId('823456789')).toBe('823 456 789')
		expect(formatLegalId('FR-1')).toBe('FR-1')
	})
})

describe('filterSuppliers', () => {
	const list = [
		supplier({ id: 'b', name: 'Zinc & Fils', email: 'zinc@fils.fr' }),
		supplier({
			id: 'a',
			name: 'Élagage Bernard',
			registration_number: '823456789',
		}),
	]

	it('sorts by name and returns everything on a blank query', () => {
		expect(filterSuppliers(list, '').map((item) => item.id)).toEqual(['a', 'b'])
	})

	it('matches accents-insensitively across the visible fields', () => {
		expect(filterSuppliers(list, 'elagage').map((item) => item.id)).toEqual([
			'a',
		])
		expect(filterSuppliers(list, 'fils.fr').map((item) => item.id)).toEqual([
			'b',
		])
		expect(filterSuppliers(list, '823456').map((item) => item.id)).toEqual([
			'a',
		])
	})
})
