import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Invoice, InvoiceKind, InvoicePayment } from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
import { PAYMENT_METHOD_LABELS } from '#/pages/invoices/lib/payment-form'
import {
	centsToEuros,
	invoiceKindLabel,
	invoiceLineTotalCents,
} from '#/pages/invoices/types'
import {
	paperIssuer,
	paperLine,
	paperRecipient,
	vatNoticeOf,
} from '#/pages/quotes/lib/paper-parties'
import type { PaperDocument } from '#/pages/quotes/ui/doc/document-paper'
import type { HistoryEntry } from '#/pages/quotes/ui/doc/history-list'
import type { DocTotals } from '#/pages/quotes/ui/doc/totals-panel'

export function invoiceHeading(kind: InvoiceKind): string {
	if (kind === 'DEPOSIT') return 'Facture d’acompte'
	if (kind === 'FINAL') return 'Facture de solde'
	if (kind === 'CREDIT_NOTE') return 'Avoir'
	return 'Facture'
}

const DAY_MS = 86_400_000

function dayStart(value: string): number {
	const day = value.slice(0, 10)
	return Date.UTC(
		Number(day.slice(0, 4)),
		Number(day.slice(5, 7)) - 1,
		Number(day.slice(8, 10)),
	)
}

export function paymentTermsLine(
	issuedAt: string,
	dueAt: string | null | undefined,
): string | null {
	if (!dueAt) return null
	const days = Math.round((dayStart(dueAt) - dayStart(issuedAt)) / DAY_MS)
	if (days < 0) return null
	if (days === 0) return 'Paiement à réception de la facture.'
	return `Délai de paiement : ${days} ${days < 2 ? 'jour' : 'jours'}.`
}

export function savedInvoiceTotals(invoice: Invoice): DocTotals {
	return {
		netCents: invoice.net_cents,
		vat: invoice.vat_breakdown.map((line) => ({
			rateBp: line.rate_bp,
			vatCents: line.vat_cents,
		})),
		grossCents: invoice.gross_cents,
	}
}

export function buildInvoicePaper(input: {
	invoice: Invoice
	customer: Customer | undefined
	context: CustomerContext | undefined
	organization: Organization
	reference?: string | null
}): PaperDocument {
	const { invoice, organization } = input
	return {
		kind: invoice.kind === 'CREDIT_NOTE' ? 'creditNote' : 'invoice',
		draft: invoice.status === 'DRAFT',
		heading: invoiceHeading(invoice.kind),
		number: invoice.number ?? null,
		title: invoice.title,
		issueDate: invoice.issued_on ?? null,
		dueDate: invoice.due_on ?? null,
		issuer: paperIssuer(organization),
		recipient: paperRecipient(input.customer, input.context),
		lines: invoice.lines.map((line) =>
			paperLine({
				id: line.id,
				label: line.label,
				quantity: line.quantity,
				unitPriceCents: line.unit_price_cents,
				vatRateBp: line.vat_rate_basis_points ?? null,
				totalCents: invoiceLineTotalCents({
					clientId: line.id,
					label: line.label,
					quantity: line.quantity,
					unitPrice: centsToEuros(line.unit_price_cents),
					vatRateBp: '',
					discountPercent: line.discount_percent,
					productId: null,
					serviceRateId: null,
				}),
			}),
		),
		totals: savedInvoiceTotals(invoice),
		showVat: organization.vat_status?.type === 'subject',
		vatNotice:
			invoice.vat_breakdown.length === 0 ? vatNoticeOf(organization) : null,
		notes: [
			invoice.kind === 'CREDIT_NOTE'
				? null
				: paymentTermsLine(
						invoice.issued_on ?? invoice.created_at,
						invoice.due_on,
					),
			invoice.kind === 'CREDIT_NOTE' ? null : invoice.legal_notice,
			invoice.notes,
		]
			.filter(Boolean)
			.join('\n'),
		reference: input.reference ?? null,
		reason: invoice.kind === 'CREDIT_NOTE' ? (invoice.reason ?? '') : '',
	}
}

export interface InvoiceActivity {
	sentAt: string | null
	reminders: readonly string[]
	internalNote: string
}

export function invoiceActivity(
	invoice: Pick<Invoice, 'sent_at' | 'internal_note' | 'events'>,
): InvoiceActivity {
	return {
		sentAt: invoice.sent_at ?? null,
		reminders: invoice.events
			.filter((event) => event.kind === 'REMINDER')
			.map((event) => event.at),
		internalNote: invoice.internal_note ?? '',
	}
}

function formatCents(cents: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(cents / 100)
}

export function buildInvoiceHistory(input: {
	invoice: Invoice
	payments: readonly InvoicePayment[]
	creditNotes: readonly Invoice[]
}): HistoryEntry[] {
	const { invoice } = input
	const entries: HistoryEntry[] = [
		{
			id: 'created',
			kind: 'created',
			detail: `${invoiceKindLabel(invoice.kind)} créée en brouillon`,
			at: invoice.created_at,
		},
	]
	if (invoice.issued_at)
		entries.push({
			id: 'issued',
			kind: 'issued',
			detail: invoice.number
				? `Facture émise sous le numéro ${invoice.number}`
				: 'Facture émise',
			at: invoice.issued_at,
		})
	if (invoice.status === 'CANCELLED')
		entries.push({
			id: 'cancelled',
			kind: 'declined',
			detail: 'Facture annulée',
			at: invoice.updated_at,
		})
	for (const event of invoice.events) {
		entries.push({
			id: `event-${event.id}`,
			kind: event.kind === 'SENT' ? 'sent' : 'note',
			detail: event.detail,
			at: event.at,
		})
	}
	for (const payment of input.payments) {
		const refund = payment.amount_cents < 0
		entries.push({
			id: `payment-${payment.id}`,
			kind: 'payment',
			detail: `${refund ? 'Remboursement' : 'Paiement'} de ${formatCents(Math.abs(payment.amount_cents))} enregistré (${PAYMENT_METHOD_LABELS[payment.method]})`,
			at: payment.created_at,
		})
	}
	for (const note of input.creditNotes) {
		entries.push({
			id: `credit-${note.id}`,
			kind: 'credit_note',
			detail: note.number ? `Avoir ${note.number} émis` : 'Avoir émis',
			at: note.created_at,
		})
	}
	return entries
}

export function creditNoteKindLabel(
	note: Pick<Invoice, 'full_cancellation'>,
): string {
	return note.full_cancellation ? 'Annulation totale' : 'Correction partielle'
}
