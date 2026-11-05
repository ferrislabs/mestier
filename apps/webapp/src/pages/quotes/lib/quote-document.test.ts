import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import { emptyQuoteLine } from '#/pages/quotes/types'
import {
	buildQuoteHistory,
	previewQuoteTotals,
	savedQuoteTotals,
} from './quote-document'

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'quote-1',
		organization_id: 'org-1',
		title: 'Salle de bain',
		status: 'DRAFT',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: 10000,
		gross_cents: 12000,
		vat_breakdown: [{ rate_bp: 2000, vat_cents: 2000 }],
		created_at: '2026-01-01T09:00:00Z',
		updated_at: '2026-01-01T09:00:00Z',
		...overrides,
	}
}

describe('savedQuoteTotals', () => {
	it('relays the totals the backend stored', () => {
		expect(savedQuoteTotals(quote())).toEqual({
			netCents: 10000,
			vat: [{ rateBp: 2000, vatCents: 2000 }],
			grossCents: 12000,
		})
	})
})

describe('previewQuoteTotals', () => {
	it('has no VAT line when the organization is not subject to VAT', () => {
		const totals = previewQuoteTotals(
			{
				title: '',
				customerId: '',
				customerContextId: '',
				lines: [{ ...emptyQuoteLine(), quantity: '2', unitPrice: '10' }],
			},
			false,
		)

		expect(totals).toEqual({ netCents: 2000, vat: [], grossCents: 2000 })
	})

	it('adds the VAT of each rate when the organization is subject', () => {
		const totals = previewQuoteTotals(
			{
				title: '',
				customerId: '',
				customerContextId: '',
				lines: [
					{
						...emptyQuoteLine(),
						quantity: '1',
						unitPrice: '100',
						vatRateBp: '2000',
					},
				],
			},
			true,
		)

		expect(totals.grossCents).toBe(12000)
		expect(totals.vat).toEqual([{ rateBp: 2000, vatCents: 2000 }])
	})
})

describe('buildQuoteHistory', () => {
	it('only lists the creation of a quote never touched', () => {
		expect(buildQuoteHistory(quote()).map((entry) => entry.kind)).toEqual([
			'created',
		])
	})

	it('reads the last status change as an event', () => {
		const entries = buildQuoteHistory(
			quote({ status: 'ACCEPTED', updated_at: '2026-01-05T09:00:00Z' }),
		)

		expect(entries.map((entry) => entry.kind)).toEqual(['created', 'accepted'])
	})
})
