import type { Schemas } from '#/api/api.client'

export type ActivityTone = 'neutral' | 'info' | 'ok' | 'warn' | 'alert'

export type QuoteRow = {
	id: string
	label: string
	title: string
	date: string
	status: string
	tone: ActivityTone
	total: number
}

export type InvoiceRow = {
	id: string
	label: string
	date: string
	status: string
	tone: ActivityTone
	total: number
}

const QUOTE_STATUS: Record<
	Schemas.QuoteStatus,
	{ label: string; tone: ActivityTone }
> = {
	DRAFT: { label: 'Brouillon', tone: 'neutral' },
	SENT: { label: 'Envoyé', tone: 'info' },
	ACCEPTED: { label: 'Accepté', tone: 'ok' },
	DECLINED: { label: 'Refusé', tone: 'alert' },
	CANCELLED: { label: 'Annulé', tone: 'neutral' },
}

const INVOICE_STATUS: Record<
	Schemas.InvoiceStatus,
	{ label: string; tone: ActivityTone }
> = {
	DRAFT: { label: 'Brouillon', tone: 'neutral' },
	ISSUED: { label: 'Émise', tone: 'info' },
	PARTIALLY_PAID: { label: 'Partiellement payée', tone: 'warn' },
	PAID: { label: 'Payée', tone: 'ok' },
	CANCELLED: { label: 'Annulée', tone: 'neutral' },
}

function newestFirst(a: { date: string }, b: { date: string }): number {
	return b.date.localeCompare(a.date)
}

export function quoteRowsOf(
	quotes: readonly Schemas.QuoteResponse[],
	customerId: string,
): QuoteRow[] {
	return quotes
		.filter((quote) => quote.customer_id === customerId)
		.map((quote) => ({
			id: quote.id,
			label: quote.reference || 'Brouillon',
			title: quote.title,
			date: quote.created_at,
			status: QUOTE_STATUS[quote.status].label,
			tone: QUOTE_STATUS[quote.status].tone,
			total: quote.gross_cents,
		}))
		.sort(newestFirst)
}

export function invoiceRowsOf(
	invoices: readonly Schemas.InvoiceResponse[],
	customerId: string,
): InvoiceRow[] {
	return invoices
		.filter((invoice) => invoice.customer_id === customerId)
		.map((invoice) => ({
			id: invoice.id,
			label: invoice.number || 'Brouillon',
			date: invoice.issued_at ?? invoice.created_at,
			status: INVOICE_STATUS[invoice.status].label,
			tone: INVOICE_STATUS[invoice.status].tone,
			total: invoice.gross_cents,
		}))
		.sort(newestFirst)
}
