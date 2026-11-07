import { describe, expect, it } from 'vitest'
import {
	hasActiveFilters,
	isStatusFilter,
	matchesClient,
	matchesOwner,
	matchesStatus,
	OWNER_ALL,
	OWNER_NONE,
	STATUS_FILTERS,
} from './quote-filter'

describe('matchesOwner', () => {
	it('lets everything through on "all"', () => {
		expect(matchesOwner(null, OWNER_ALL)).toBe(true)
		expect(matchesOwner('m1', OWNER_ALL)).toBe(true)
	})

	it('keeps the quotes without owner on "none"', () => {
		expect(matchesOwner(null, OWNER_NONE)).toBe(true)
		expect(matchesOwner('m1', OWNER_NONE)).toBe(false)
	})

	it('keeps the quotes of one owner', () => {
		expect(matchesOwner('m1', 'm1')).toBe(true)
		expect(matchesOwner('m2', 'm1')).toBe(false)
		expect(matchesOwner(null, 'm1')).toBe(false)
	})
})

describe('matchesStatus', () => {
	it('lets everything through on "all"', () => {
		expect(matchesStatus('expired', 'all')).toBe(true)
	})

	it('matches the display status, so expired is not sent', () => {
		expect(matchesStatus('expired', 'sent')).toBe(false)
		expect(matchesStatus('expired', 'expired')).toBe(true)
		expect(matchesStatus('superseded', 'superseded')).toBe(true)
	})
})

describe('matchesClient', () => {
	it('lets everything through without a client filter', () => {
		expect(matchesClient('c1', null)).toBe(true)
	})

	it('keeps one client', () => {
		expect(matchesClient('c1', 'c1')).toBe(true)
		expect(matchesClient('c2', 'c1')).toBe(false)
	})
})

describe('status filters', () => {
	it('offers every display status after "all"', () => {
		expect(STATUS_FILTERS).toEqual([
			'all',
			'draft',
			'sent',
			'accepted',
			'declined',
			'expired',
			'superseded',
			'cancelled',
		])
		expect(isStatusFilter('expired')).toBe(true)
		expect(isStatusFilter('SENT')).toBe(false)
	})
})

describe('hasActiveFilters', () => {
	it('is false on the defaults', () => {
		expect(
			hasActiveFilters({ query: '', status: 'all', owner: OWNER_ALL }),
		).toBe(false)
	})

	it('is true as soon as one filter is set', () => {
		expect(
			hasActiveFilters({ query: ' x ', status: 'all', owner: OWNER_ALL }),
		).toBe(true)
		expect(
			hasActiveFilters({ query: '', status: 'sent', owner: OWNER_ALL }),
		).toBe(true)
		expect(
			hasActiveFilters({ query: '', status: 'all', owner: OWNER_NONE }),
		).toBe(true)
	})
})
