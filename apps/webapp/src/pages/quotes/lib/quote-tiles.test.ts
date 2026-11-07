import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import { EMPTY_TERMS, type QuoteTerms } from './quote-terms'
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
		deposit_percent: '0',
		is_expired: false,
		recurring: [],
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

describe('computeQuoteTiles on a dated view', () => {
	const TODAY = '2026-11-07'
	const view = (terms: Record<string, Partial<QuoteTerms>>) => ({
		today: TODAY,
		termsOf: (id: string) => ({ ...EMPTY_TERMS, ...terms[id] }),
	})
	const dated = (
		status: Quote['status'],
		id: string,
		updatedAt: string,
		netCents = 1000,
	): Quote => ({ ...quote(status, netCents), id, updated_at: updatedAt })

	it('does not count an expired quote as awaiting an answer', () => {
		const tiles = computeQuoteTiles(
			[
				quote('SENT', 2000),
				{ ...quote('SENT', 500), id: 'old', is_expired: true },
			],
			view({ old: { validUntil: '2026-10-01' } }),
		)

		expect(tiles.pending).toEqual({ count: 1, netCents: 2000 })
	})

	it('counts an expired quote as a lost one in the acceptance rate', () => {
		const tiles = computeQuoteTiles(
			[
				dated('ACCEPTED', 'won', '2026-11-01T00:00:00Z'),
				{ ...quote('SENT'), id: 'old', is_expired: true },
			],
			view({ old: { validUntil: '2026-11-01' } }),
		)

		expect(tiles.acceptanceRate).toBe(50)
	})

	it('only counts answers of the last 90 days', () => {
		const tiles = computeQuoteTiles(
			[
				dated('ACCEPTED', 'recent', '2026-10-20T00:00:00Z', 3000),
				dated('ACCEPTED', 'ancient', '2026-01-02T00:00:00Z', 9000),
				dated('DECLINED', 'ancient-no', '2026-01-02T00:00:00Z'),
			],
			view({}),
		)

		expect(tiles.accepted).toEqual({ count: 1, netCents: 3000 })
		expect(tiles.acceptanceRate).toBe(100)
	})

	it('prefers the answer date the backend recorded', () => {
		const tiles = computeQuoteTiles(
			[dated('ACCEPTED', 'q', '2026-01-02T00:00:00Z')],
			view({ q: { respondedAt: '2026-11-02' } }),
		)

		expect(tiles.accepted.count).toBe(1)
	})

	it('does not count a replaced quote as pending', () => {
		const tiles = computeQuoteTiles(
			[{ ...quote('SUPERSEDED'), id: 'old' }],
			view({ old: { supersededBy: { id: 'n', label: 'DEV-2' } } }),
		)

		expect(tiles.pending.count).toBe(0)
	})
})
