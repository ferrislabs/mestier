import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import { computeQuoteTiles } from './quote-tiles'

function quote(status: Quote['status'], netCents = 1000): Quote {
	return {
		id: `${status}-${netCents}`,
		organization_id: 'org-1',
		title: 'Devis',
		status,
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: netCents,
		gross_cents: netCents,
		vat_breakdown: [],
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
}

describe('computeQuoteTiles', () => {
	it('has no acceptance rate while nothing is decided', () => {
		expect(
			computeQuoteTiles([quote('DRAFT'), quote('SENT')]).acceptanceRate,
		).toBeNull()
	})

	it('counts pending and accepted quotes with their net amount', () => {
		const tiles = computeQuoteTiles([
			quote('SENT', 2000),
			quote('SENT', 500),
			quote('ACCEPTED', 4000),
			quote('DRAFT'),
		])

		expect(tiles.pending).toEqual({ count: 2, netCents: 2500 })
		expect(tiles.accepted).toEqual({ count: 1, netCents: 4000 })
		expect(tiles.drafts).toBe(1)
	})

	it('rates acceptance over accepted and declined quotes only', () => {
		const tiles = computeQuoteTiles([
			quote('ACCEPTED'),
			quote('ACCEPTED', 1100),
			quote('DECLINED'),
			quote('CANCELLED'),
		])

		expect(tiles.acceptanceRate).toBe(67)
	})
})
