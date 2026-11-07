import { describe, expect, it } from 'vitest'
import {
	customerStats,
	type StatInvoice,
	type StatQuote,
} from '#/pages/customers/lib/stats'

const NOW = new Date('2026-11-07T10:00:00Z').getTime()

function invoice(overrides: Partial<StatInvoice>): StatInvoice {
	return {
		customer_id: 'c1',
		kind: 'STANDARD',
		status: 'ISSUED',
		net_cents: 100_00,
		gross_cents: 120_00,
		issued_at: '2026-10-01T00:00:00Z',
		created_at: '2026-09-30T00:00:00Z',
		due_at: '2026-11-30T00:00:00Z',
		...overrides,
	}
}

function quote(overrides: Partial<StatQuote>): StatQuote {
	return { customer_id: 'c1', created_at: '2026-08-01T00:00:00Z', ...overrides }
}

describe('customerStats', () => {
	it('sums the net of issued invoices of the last 12 months minus credit notes', () => {
		const stats = customerStats({
			customerId: 'c1',
			invoices: [
				invoice({ net_cents: 100_00 }),
				invoice({ status: 'PAID', net_cents: 50_00 }),
				invoice({ status: 'DRAFT', net_cents: 999_00 }),
				invoice({ status: 'CANCELLED', net_cents: 999_00 }),
				invoice({ issued_at: '2025-01-01T00:00:00Z', net_cents: 999_00 }),
				invoice({ customer_id: 'c2', net_cents: 999_00 }),
				invoice({ kind: 'CREDIT_NOTE', net_cents: 30_00 }),
			],
			quotes: [],
			outstandingCents: 0,
			now: NOW,
		})
		expect(stats.revenue12m).toBe(120_00)
	})

	it('never goes below zero', () => {
		const stats = customerStats({
			customerId: 'c1',
			invoices: [invoice({ kind: 'CREDIT_NOTE', net_cents: 30_00 })],
			quotes: [],
			outstandingCents: 0,
			now: NOW,
		})
		expect(stats.revenue12m).toBe(0)
	})

	it('counts overdue invoices, capped by the outstanding balance', () => {
		const invoices = [
			invoice({ due_at: '2026-10-01T00:00:00Z', gross_cents: 120_00 }),
			invoice({
				status: 'PARTIALLY_PAID',
				due_at: '2026-10-05T00:00:00Z',
				gross_cents: 240_00,
			}),
			invoice({ due_at: '2026-12-01T00:00:00Z', gross_cents: 500_00 }),
		]
		const base = { customerId: 'c1', invoices, quotes: [], now: NOW }
		expect(customerStats({ ...base, outstandingCents: 1000_00 }).overdue).toBe(
			360_00,
		)
		expect(customerStats({ ...base, outstandingCents: 200_00 }).overdue).toBe(
			200_00,
		)
		expect(customerStats({ ...base, outstandingCents: 0 }).overdue).toBe(0)
	})

	it('picks the latest activity among quotes and issued invoices', () => {
		const stats = customerStats({
			customerId: 'c1',
			invoices: [
				invoice({ issued_at: '2026-10-01T00:00:00Z' }),
				invoice({
					status: 'DRAFT',
					issued_at: null,
					created_at: '2026-11-01T00:00:00Z',
				}),
			],
			quotes: [
				quote({ created_at: '2026-10-15T00:00:00Z' }),
				quote({ customer_id: 'c2', created_at: '2026-11-05T00:00:00Z' }),
			],
			outstandingCents: 0,
			now: NOW,
		})
		expect(stats.lastActivity).toBe('2026-10-15T00:00:00Z')
	})

	it('has no activity without documents', () => {
		const stats = customerStats({
			customerId: 'c1',
			invoices: [],
			quotes: [],
			outstandingCents: 0,
			now: NOW,
		})
		expect(stats).toEqual({ revenue12m: 0, overdue: 0, lastActivity: null })
	})
})
