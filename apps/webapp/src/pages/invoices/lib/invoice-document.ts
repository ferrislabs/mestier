import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Invoice, InvoiceKind, InvoicePayment } from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
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
		title: '',
		issueDate: invoice.issued_at ?? null,
		dueDate: invoice.due_at ?? null,
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
				}),
			}),
		),
		totals: savedInvoiceTotals(invoice),
		showVat: organization.vat_status?.type === 'subject',
		vatNotice:
			invoice.vat_breakdown.length === 0 ? vatNoticeOf(organization) : null,
		notes: invoice.notes ?? '',
		reference: input.reference ?? null,
		reason: invoice.kind === 'CREDIT_NOTE' ? (invoice.notes ?? '') : '',
	}
}

export interface InvoiceActivity {
	sentAt: string | null
	reminders: readonly string[]
}

export function buildInvoiceHistory(input: {
	invoice: Invoice
	payments: readonly InvoicePayment[]
	creditNotes: readonly Invoice[]
	activity: InvoiceActivity
}): HistoryEntry[] {
	const { invoice, activity } = input
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
	if (activity.sentAt)
		entries.push({
			id: 'sent',
			kind: 'sent',
			detail: 'Facture envoyée au client',
			at: activity.sentAt,
		})
	activity.reminders.forEach((at, index) => {
		entries.push({
			id: `reminder-${index}`,
			kind: 'note',
			detail: 'Relance enregistrée',
			at,
		})
	})
	for (const payment of input.payments) {
		entries.push({
			id: `payment-${payment.id}`,
			kind: 'payment',
			detail: `Paiement de ${new Intl.NumberFormat('fr-FR', {
				style: 'currency',
				currency: 'EUR',
			}).format(payment.amount_cents / 100)} enregistré (${payment.method})`,
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
	note: Pick<Invoice, 'gross_cents'>,
	source: Pick<Invoice, 'gross_cents'> | undefined,
): string {
	return source && note.gross_cents >= source.gross_cents
		? 'Annulation totale'
		: 'Correction partielle'
}
