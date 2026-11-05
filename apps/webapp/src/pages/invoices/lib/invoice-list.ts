import type { Invoice } from '#/hooks/use-invoices'
import {
	INVOICE_STATE_LABELS,
	type InvoiceDisplayState,
	invoiceDisplayState,
} from '#/pages/invoices/types'

export type InvoiceStateFilter =
	| 'all'
	| 'unpaid'
	| 'overdue'
	| 'paid'
	| 'draft'
	| 'cancelled'

export function matchesStateFilter(
	state: InvoiceDisplayState,
	filter: string,
): boolean {
	if (filter === 'all') return true
	if (filter === 'unpaid') return state === 'unpaid' || state === 'partial'
	return state === filter
}

export function invoiceStateCounts(
	invoices: readonly Invoice[],
	now: Date = new Date(),
): Record<InvoiceStateFilter, number> {
	const states = invoices.map((invoice) =>
		invoiceDisplayState(invoice, null, now),
	)
	const count = (filter: InvoiceStateFilter) =>
		states.filter((state) => matchesStateFilter(state, filter)).length
	return {
		all: states.length,
		unpaid: count('unpaid'),
		overdue: count('overdue'),
		paid: count('paid'),
		draft: count('draft'),
		cancelled: count('cancelled'),
	}
}

export interface InvoiceTiles {
	toCollect: { amountCents: number | null; count: number }
	overdue: number
	paid: number
	drafts: { count: number; grossCents: number }
}

export function computeInvoiceTiles(
	invoices: readonly Invoice[],
	outstandingTotalCents: number | null,
	now: Date = new Date(),
): InvoiceTiles {
	const states = invoices.map((invoice) => ({
		invoice,
		state: invoiceDisplayState(invoice, null, now),
	}))
	const drafts = states.filter(({ state }) => state === 'draft')
	return {
		toCollect: {
			amountCents: outstandingTotalCents,
			count: states.filter(({ state }) => matchesStateFilter(state, 'unpaid'))
				.length,
		},
		overdue: states.filter(({ state }) => state === 'overdue').length,
		paid: states.filter(({ state }) => state === 'paid').length,
		drafts: {
			count: drafts.length,
			grossCents: drafts.reduce(
				(sum, { invoice }) => sum + invoice.gross_cents,
				0,
			),
		},
	}
}

export function invoiceMatchesSearch(
	invoice: Invoice,
	wanted: string,
	customerName: string,
): boolean {
	return (
		invoice.id.toLowerCase().includes(wanted) ||
		(invoice.number?.toLowerCase().includes(wanted) ?? false) ||
		INVOICE_STATE_LABELS[invoiceDisplayState(invoice)]
			.toLowerCase()
			.includes(wanted) ||
		customerName.includes(wanted)
	)
}
