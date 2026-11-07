import { describe, expect, it } from 'vitest'
import type { Invoice } from '#/hooks/use-invoices'
import {
	computeInvoiceTiles,
	excludeInvoices,
	invoiceMatchesAccount,
	invoiceMatchesSearch,
	invoiceStateCounts,
	matchesStateFilter,
} from './invoice-list'

const NOW = new Date('2026-06-10T12:00:00Z')

function invoice(overrides: Partial<Invoice> = {}): Invoice {
	return {
		id: 'invoice-1',
		organization_id: 'org-1',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		kind: 'STANDARD',
		status: 'ISSUED',
		number: 'FAC-001',
		gross_cents: 12000,
		net_cents: 10000,
		lines: [],
		vat_breakdown: [],
		due_at: '2026-07-01T00:00:00Z',
		created_at: '2026-06-01T00:00:00Z',
		updated_at: '2026-06-01T00:00:00Z',
		...overrides,
	}
}

describe('matchesStateFilter', () => {
	it('groups unpaid and partially paid invoices under "À payer"', () => {
		expect(matchesStateFilter('unpaid', 'unpaid')).toBe(true)
		expect(matchesStateFilter('partial', 'unpaid')).toBe(true)
		expect(matchesStateFilter('overdue', 'unpaid')).toBe(false)
	})

	it('matches any state with no filter', () => {
		expect(matchesStateFilter('cancelled', 'all')).toBe(true)
	})
})

describe('invoiceStateCounts', () => {
	it('counts each state, an overdue invoice only as overdue', () => {
		const counts = invoiceStateCounts(
			[
				invoice(),
				invoice({ due_at: '2026-06-01T00:00:00Z' }),
				invoice({ status: 'PAID' }),
				invoice({ status: 'DRAFT', number: null }),
				invoice({ status: 'CANCELLED' }),
			],
			NOW,
		)

		expect(counts).toEqual({
			all: 5,
			unpaid: 1,
			overdue: 1,
			paid: 1,
			draft: 1,
			cancelled: 1,
		})
	})
})

describe('computeInvoiceTiles', () => {
	it('relays the backend outstanding total and sums the stored draft totals', () => {
		const tiles = computeInvoiceTiles(
			[
				invoice(),
				invoice({ status: 'PARTIALLY_PAID' }),
				invoice({ due_at: '2026-06-01T00:00:00Z' }),
				invoice({ status: 'PAID' }),
				invoice({ status: 'DRAFT', gross_cents: 500 }),
				invoice({ status: 'DRAFT', gross_cents: 250 }),
			],
			42000,
			NOW,
		)

		expect(tiles.toCollect).toEqual({ amountCents: 42000, count: 2 })
		expect(tiles.overdue).toBe(1)
		expect(tiles.paid).toBe(1)
		expect(tiles.drafts).toEqual({ count: 2, grossCents: 750 })
	})
})

describe('invoiceMatchesSearch', () => {
	it('finds an invoice by number or customer', () => {
		expect(invoiceMatchesSearch(invoice(), 'fac-001', '')).toBe(true)
		expect(invoiceMatchesSearch(invoice(), 'dupont', 'menuiserie dupont')).toBe(
			true,
		)
		expect(invoiceMatchesSearch(invoice(), 'zzz', 'dupont')).toBe(false)
	})
})

describe('excludeInvoices', () => {
	it('drops the invoices whose id is hidden', () => {
		const list = [invoice({ id: 'a' }), invoice({ id: 'b' })]

		expect(excludeInvoices(list, ['b']).map((item) => item.id)).toEqual(['a'])
		expect(excludeInvoices(list, [])).toHaveLength(2)
	})
})

describe('invoiceMatchesAccount', () => {
	it('keeps every invoice without a customer filter', () => {
		expect(invoiceMatchesAccount(invoice(), null)).toBe(true)
	})

	it('keeps only the invoices of the chosen customer', () => {
		expect(invoiceMatchesAccount(invoice(), 'customer-1')).toBe(true)
		expect(invoiceMatchesAccount(invoice(), 'customer-2')).toBe(false)
	})
})
