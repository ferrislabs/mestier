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

export type CreditNoteRow = {
	id: string
	label: string
	sourceLabel: string
	date: string
	kind: string
	total: number
}

export type PaymentRow = {
	id: string
	invoiceId: string
	target: string
	date: string
	detail: string
	amount: number
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
		.filter(
			(invoice) =>
				invoice.customer_id === customerId && invoice.kind !== 'CREDIT_NOTE',
		)
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

export function creditNoteRowsOf(
	invoices: readonly Schemas.InvoiceResponse[],
	customerId: string,
): CreditNoteRow[] {
	return invoices
		.filter(
			(invoice) =>
				invoice.customer_id === customerId && invoice.kind === 'CREDIT_NOTE',
		)
		.map((note) => {
			const source = invoices.find((item) => item.id === note.source_invoice_id)
			return {
				id: note.id,
				label: note.number || 'Brouillon',
				sourceLabel: source
					? source.number || 'Brouillon'
					: 'Facture introuvable',
				date: note.issued_at ?? note.created_at,
				kind:
					source && note.gross_cents >= source.gross_cents
						? 'Annulation totale'
						: 'Avoir partiel',
				total: note.gross_cents,
			}
		})
		.sort(newestFirst)
}

export function paymentRowsOf(
	entries: readonly {
		invoice: Schemas.InvoiceResponse
		payments: readonly Schemas.InvoicePaymentResponse[]
	}[],
): PaymentRow[] {
	return entries
		.flatMap(({ invoice, payments }) =>
			payments
				.filter((payment) => !payment.deleted_at)
				.map((payment) => ({
					id: payment.id,
					invoiceId: invoice.id,
					target: invoice.number || 'Brouillon',
					date: payment.paid_on,
					detail: [payment.method, payment.reference]
						.filter(Boolean)
						.join(' · '),
					amount: payment.amount_cents,
				})),
		)
		.sort(newestFirst)
}
