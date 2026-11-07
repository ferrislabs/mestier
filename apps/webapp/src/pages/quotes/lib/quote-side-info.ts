import type { Quote } from '#/hooks/use-quotes'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { type ValidityNote, validityNote } from '#/pages/quotes/lib/quote-draft'
import {
	type QuoteDisplayStatus,
	quoteDisplayStatus,
} from '#/pages/quotes/lib/quote-status'
import type { QuoteLink, QuoteTerms } from '#/pages/quotes/lib/quote-terms'

export interface QuoteSideInfo {
	displayStatus: QuoteDisplayStatus
	validity: ValidityNote
	answerNote: string | null
	declineReason: string
	ownerName: string | null
	revisionOf: QuoteLink | null
	supersededBy: QuoteLink | null
	dealId: string | null
}

function answerNoteOf(quote: Quote, terms: QuoteTerms): string | null {
	const answeredOn = formatDocumentDate(
		terms.respondedAt ?? quote.updated_at.slice(0, 10),
	)
	if (quote.status === 'ACCEPTED')
		return `Accepté le ${answeredOn}${terms.signer ? ` par ${terms.signer}` : ''}`
	if (quote.status === 'DECLINED') return `Refusé le ${answeredOn}`
	return null
}

export function buildQuoteSideInfo(
	quote: Quote,
	terms: QuoteTerms,
	owners: readonly { id: string; name: string }[],
	today: string,
): QuoteSideInfo {
	return {
		displayStatus: quoteDisplayStatus(quote),
		validity: validityNote(
			quote.status,
			terms.validUntil,
			today,
			quote.is_expired,
		),
		answerNote: answerNoteOf(quote, terms),
		declineReason: quote.status === 'DECLINED' ? terms.declineReason : '',
		ownerName: owners.find((owner) => owner.id === terms.ownerId)?.name ?? null,
		revisionOf: terms.revisionOf,
		supersededBy: terms.supersededBy,
		dealId: terms.dealId,
	}
}
