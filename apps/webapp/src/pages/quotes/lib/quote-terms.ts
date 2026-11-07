import type { Quote } from '#/hooks/use-quotes'
import {
	type BillingFrequency,
	frequencyOfApi,
} from '#/pages/quotes/lib/billing-frequency'

export interface QuoteLineTerms {
	discountPercent: number
	frequency: BillingFrequency
}

export interface QuoteLink {
	id: string
	label: string
}

export interface QuoteLinks {
	revisionOf: QuoteLink | null
	supersededBy: QuoteLink | null
}

export interface QuoteTerms {
	ownerId: string | null
	validUntil: string | null
	depositPercent: number
	notes: string
	lines: QuoteLineTerms[]
	signer: string
	declineReason: string
	respondedAt: string | null
	revisionOf: QuoteLink | null
	supersededBy: QuoteLink | null
	dealId: string | null
}

export const EMPTY_TERMS: QuoteTerms = {
	ownerId: null,
	validUntil: null,
	depositPercent: 0,
	notes: '',
	lines: [],
	signer: '',
	declineReason: '',
	respondedAt: null,
	revisionOf: null,
	supersededBy: null,
	dealId: null,
}

export const NO_LINKS: QuoteLinks = { revisionOf: null, supersededBy: null }

const ORIGIN_LABEL = 'Devis d’origine'
const NEXT_LABEL = 'Nouvelle version'

export function termsOfQuote(
	quote: Quote,
	links: QuoteLinks = NO_LINKS,
): QuoteTerms {
	return {
		ownerId: quote.owner_member_id ?? null,
		validUntil: quote.valid_until ?? null,
		depositPercent: Number(quote.deposit_percent),
		notes: quote.notes ?? '',
		lines: quote.lines.map((line) => ({
			discountPercent: Number(line.discount_percent),
			frequency: frequencyOfApi(line.billing_frequency),
		})),
		signer: quote.responded_by ?? '',
		declineReason: quote.decline_reason ?? '',
		respondedAt: quote.responded_at?.slice(0, 10) ?? null,
		revisionOf:
			links.revisionOf ??
			(quote.revision_of_id
				? { id: quote.revision_of_id, label: ORIGIN_LABEL }
				: null),
		supersededBy:
			links.supersededBy ??
			(quote.superseded_by_id
				? { id: quote.superseded_by_id, label: NEXT_LABEL }
				: null),
		dealId: quote.deal_id ?? null,
	}
}

export function quoteLinkOf(
	quote: Pick<Quote, 'id' | 'reference'> | undefined,
	fallback: QuoteLink | null,
): QuoteLink | null {
	if (!quote) return fallback
	return { id: quote.id, label: quote.reference ?? 'Brouillon' }
}

export function lineTermsAt(
	terms: Pick<QuoteTerms, 'lines'>,
	index: number,
): QuoteLineTerms {
	return terms.lines[index] ?? { discountPercent: 0, frequency: 'one_shot' }
}
