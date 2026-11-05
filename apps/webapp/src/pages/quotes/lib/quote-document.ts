import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import {
	paperIssuer,
	paperLine,
	paperRecipient,
	vatNoticeOf,
} from '#/pages/quotes/lib/paper-parties'
import {
	eurosToCents,
	type QuoteFormValues,
	quoteLinesGrossCents,
	quoteLinesVatBreakdown,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import type { PaperDocument } from '#/pages/quotes/ui/doc/document-paper'
import type { HistoryEntry } from '#/pages/quotes/ui/doc/history-list'
import type { DocTotals } from '#/pages/quotes/ui/doc/totals-panel'

export function savedQuoteTotals(quote: Quote): DocTotals {
	return {
		netCents: quote.net_cents,
		vat: quote.vat_breakdown.map((line) => ({
			rateBp: line.rate_bp,
			vatCents: line.vat_cents,
		})),
		grossCents: quote.gross_cents,
	}
}

export function previewQuoteTotals(
	values: QuoteFormValues,
	vatEnabled: boolean,
): DocTotals {
	const netCents = values.lines.reduce(
		(sum, line) => sum + quoteLineTotalCents(line),
		0,
	)
	const vat = quoteLinesVatBreakdown(values.lines, vatEnabled)
	return {
		netCents,
		vat: vat.map((line) => ({ rateBp: line.rateBp, vatCents: line.vatCents })),
		grossCents: quoteLinesGrossCents(netCents, vat),
	}
}

export function buildQuotePaper(input: {
	quote: Quote
	values: QuoteFormValues
	customer: Customer | undefined
	context: CustomerContext | undefined
	organization: Organization
	totals: DocTotals
}): PaperDocument {
	const { quote, values, organization } = input
	return {
		kind: 'quote',
		draft: quote.status === 'DRAFT',
		heading: 'Devis',
		number: quote.reference ?? null,
		title: values.title,
		issueDate: quote.created_at,
		dueDate: null,
		issuer: paperIssuer(organization),
		recipient: paperRecipient(input.customer, input.context),
		lines: values.lines.map((line) =>
			paperLine({
				id: line.clientId,
				label: line.label,
				notes: line.notes,
				quantity: line.quantity,
				unit: line.unit,
				unitPriceCents: eurosToCents(line.unitPrice),
				vatRateBp: line.vatRateBp === '' ? null : Number(line.vatRateBp),
				totalCents: quoteLineTotalCents(line),
			}),
		),
		totals: input.totals,
		showVat: organization.vat_status?.type === 'subject',
		vatNotice: vatNoticeOf(organization),
		notes: '',
		reference: null,
		reason: '',
	}
}

const STATUS_EVENTS: Partial<
	Record<Quote['status'], { kind: HistoryEntry['kind']; detail: string }>
> = {
	SENT: { kind: 'sent', detail: 'Devis envoyé au client' },
	ACCEPTED: { kind: 'accepted', detail: 'Devis accepté par le client' },
	DECLINED: { kind: 'declined', detail: 'Devis refusé par le client' },
}

export function buildQuoteHistory(quote: Quote): HistoryEntry[] {
	const entries: HistoryEntry[] = [
		{
			id: 'created',
			kind: 'created',
			detail: 'Devis créé',
			at: quote.created_at,
		},
	]
	if (quote.updated_at !== quote.created_at) {
		const event = STATUS_EVENTS[quote.status]
		entries.push({
			id: 'updated',
			kind: event?.kind ?? 'updated',
			detail: event?.detail ?? 'Dernière modification',
			at: quote.updated_at,
		})
	}
	return entries
}
