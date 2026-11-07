import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import {
	paperIssuer,
	paperLine,
	paperRecipient,
	vatNoticeOf,
} from '#/pages/quotes/lib/paper-parties'
import { parsePercent } from '#/pages/quotes/lib/quote-draft'
import type { QuoteExtras } from '#/pages/quotes/lib/quote-extras'
import {
	billingAddressLines,
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
		validUntil: values.validUntil === '' ? null : values.validUntil,
		siteAddressLines: siteAddressLines(input.context),
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
				discountPercent: parsePercent(line.discountPercent) ?? 0,
				frequency: line.frequency,
			}),
		),
		totals: input.totals,
		showVat: organization.vat_status?.type === 'subject',
		vatNotice: vatNoticeOf(organization),
		notes: values.notes.trim(),
		reference: null,
		reason: '',
	}
}

function siteAddressLines(context: CustomerContext | undefined): string[] {
	if (!context) return []
	return [context.label.trim(), ...billingAddressLines(context)].filter(
		(line) => line !== '',
	)
}

function statusEvent(
	quote: Quote,
	extras: QuoteExtras,
): { kind: HistoryEntry['kind']; detail: string } | undefined {
	switch (quote.status) {
		case 'SENT':
			return { kind: 'sent', detail: 'Devis envoyé au client' }
		case 'ACCEPTED':
			return {
				kind: 'accepted',
				detail: extras.signer
					? `Devis accepté par ${extras.signer}`
					: 'Devis accepté par le client',
			}
		case 'DECLINED':
			return {
				kind: 'declined',
				detail: extras.declineReason
					? `Devis refusé : ${extras.declineReason}`
					: 'Devis refusé par le client',
			}
		default:
			return undefined
	}
}

export function buildQuoteHistory(
	quote: Quote,
	extras: QuoteExtras,
): HistoryEntry[] {
	const entries: HistoryEntry[] = [
		{
			id: 'created',
			kind: 'created',
			detail: 'Devis créé',
			at: quote.created_at,
		},
	]
	if (extras.revisionOf) {
		entries.push({
			id: 'revision-of',
			kind: 'revised',
			detail: `Révision de ${extras.revisionOf.label}`,
			at: quote.created_at,
		})
	}
	if (quote.updated_at !== quote.created_at) {
		const event = statusEvent(quote, extras)
		entries.push({
			id: 'updated',
			kind: event?.kind ?? 'updated',
			detail: event?.detail ?? 'Dernière modification',
			at: quote.updated_at,
		})
	}
	for (const event of extras.events) {
		entries.push({
			id: event.id,
			kind: event.kind,
			detail: event.detail,
			at: event.at,
		})
	}
	return entries.sort((a, b) => a.at.localeCompare(b.at))
}
