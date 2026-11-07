import type { Schemas } from '#/api/api.client'

export type StatInvoice = Pick<
	Schemas.InvoiceResponse,
	| 'customer_id'
	| 'kind'
	| 'status'
	| 'net_cents'
	| 'gross_cents'
	| 'issued_at'
	| 'created_at'
	| 'due_at'
>

export type StatQuote = Pick<
	Schemas.QuoteResponse,
	'customer_id' | 'created_at'
>

export type CustomerStats = {
	revenue12m: number
	overdue: number
	lastActivity: string | null
}

const YEAR_MS = 365 * 86_400_000

function isIssued(invoice: StatInvoice): boolean {
	return invoice.status !== 'DRAFT' && invoice.status !== 'CANCELLED'
}

function isOpen(invoice: StatInvoice): boolean {
	return invoice.status === 'ISSUED' || invoice.status === 'PARTIALLY_PAID'
}

function timeOf(value: string | null | undefined): number {
	return value ? new Date(value).getTime() : Number.NaN
}

export function customerStats({
	customerId,
	invoices,
	quotes,
	outstandingCents,
	now,
}: {
	customerId: string
	invoices: readonly StatInvoice[]
	quotes: readonly StatQuote[]
	outstandingCents: number
	now: number
}): CustomerStats {
	const own = invoices.filter(
		(invoice) => invoice.customer_id === customerId && isIssued(invoice),
	)
	const since = now - YEAR_MS
	let revenue = 0
	let overdue = 0
	const dates: string[] = []
	for (const invoice of own) {
		const issued = invoice.issued_at ?? invoice.created_at
		if (timeOf(issued) >= since)
			revenue +=
				invoice.kind === 'CREDIT_NOTE' ? -invoice.net_cents : invoice.net_cents
		if (invoice.kind === 'CREDIT_NOTE') continue
		dates.push(issued)
		if (isOpen(invoice) && timeOf(invoice.due_at) < now)
			overdue += invoice.gross_cents
	}
	for (const quote of quotes)
		if (quote.customer_id === customerId) dates.push(quote.created_at)
	return {
		revenue12m: Math.max(0, revenue),
		overdue: Math.min(overdue, Math.max(0, outstandingCents)),
		lastActivity: dates.sort().at(-1) ?? null,
	}
}
