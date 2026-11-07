import { describe, expect, it } from 'vitest'
import type { Invoice } from '#/hooks/use-invoices'
import { invoiceFixture } from '#/pages/invoices/test-support/invoice-fixture'
import {
	computeInvoiceTiles,
	invoiceMatchesAccount,
	invoiceMatchesSearch,
	invoiceStateCounts,
	matchesStateFilter,
} from './invoice-list'

function invoice(overrides: Partial<Invoice> = {}): Invoice {
	return invoiceFixture({ due_at: '2026-07-01T00:00:00Z', ...overrides })
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
	it('counts each state the backend decided', () => {
		const counts = invoiceStateCounts([
			invoice(),
			invoice({ display_state: 'OVERDUE' }),
			invoice({ display_state: 'PAID' }),
			invoice({ display_state: 'DRAFT', number: null }),
			invoice({ display_state: 'CANCELLED' }),
			invoice({ display_state: 'REFUND_DUE', balance_cents: -2000 }),
		])

		expect(counts).toEqual({
			all: 6,
			unpaid: 1,
			overdue: 1,
			paid: 1,
			draft: 1,
			cancelled: 1,
			refund_due: 1,
		})
	})
})

describe('computeInvoiceTiles', () => {
	it('relays the backend outstanding total and sums the stored draft totals', () => {
		const tiles = computeInvoiceTiles(
			[
				invoice(),
				invoice({ display_state: 'PARTIAL' }),
				invoice({ display_state: 'OVERDUE' }),
				invoice({ display_state: 'PAID' }),
				invoice({ display_state: 'DRAFT', gross_cents: 500 }),
				invoice({ display_state: 'DRAFT', gross_cents: 250 }),
			],
			42000,
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

describe('invoiceMatchesAccount', () => {
	it('keeps every invoice without a customer filter', () => {
		expect(invoiceMatchesAccount(invoice(), null)).toBe(true)
	})

	it('keeps only the invoices of the chosen customer', () => {
		expect(invoiceMatchesAccount(invoice(), 'customer-1')).toBe(true)
		expect(invoiceMatchesAccount(invoice(), 'customer-2')).toBe(false)
	})
})
