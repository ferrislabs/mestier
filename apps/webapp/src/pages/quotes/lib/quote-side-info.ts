import type { Quote } from '#/hooks/use-quotes'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { type ValidityNote, validityNote } from '#/pages/quotes/lib/quote-draft'
import type { QuoteExtras, QuoteLink } from '#/pages/quotes/lib/quote-extras'
import {
	type QuoteDisplayStatus,
	quoteDisplayStatus,
} from '#/pages/quotes/lib/quote-status'

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

function answerNoteOf(quote: Quote, extras: QuoteExtras): string | null {
	const answeredOn = formatDocumentDate(
		extras.respondedAt ?? quote.updated_at.slice(0, 10),
	)
	if (quote.status === 'ACCEPTED')
		return `Accepté le ${answeredOn}${extras.signer ? ` par ${extras.signer}` : ''}`
	if (quote.status === 'DECLINED') return `Refusé le ${answeredOn}`
	return null
}

export function buildQuoteSideInfo(
	quote: Quote,
	extras: QuoteExtras,
	owners: readonly { id: string; name: string }[],
	today: string,
): QuoteSideInfo {
	return {
		displayStatus: quoteDisplayStatus(quote.status, extras, today),
		validity: validityNote(quote.status, extras.validUntil, today),
		answerNote: answerNoteOf(quote, extras),
		declineReason: quote.status === 'DECLINED' ? extras.declineReason : '',
		ownerName:
			owners.find((owner) => owner.id === extras.ownerId)?.name ?? null,
		revisionOf: extras.revisionOf,
		supersededBy: extras.supersededBy,
		dealId: extras.dealId,
	}
}
