import type { Quote } from '#/hooks/use-quotes'
import { addDays } from '#/pages/quotes/lib/quote-draft'
import type { QuoteExtras } from '#/pages/quotes/lib/quote-extras'
import { quoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

export const ACCEPTANCE_WINDOW_DAYS = 90

export interface QuoteTile {
	count: number
	netCents: number
}

export interface QuoteTiles {
	pending: QuoteTile
	accepted: QuoteTile
	acceptanceRate: number | null
	drafts: number
}

export interface QuoteTilesView {
	today: string
	extrasOf: (quoteId: string) => QuoteExtras
}

function tileOf(quotes: readonly Quote[]): QuoteTile {
	return {
		count: quotes.length,
		netCents: quotes.reduce((sum, quote) => sum + quote.net_cents, 0),
	}
}

function answeredOn(quote: Quote, extras: QuoteExtras): string {
	return extras.respondedAt ?? quote.updated_at.slice(0, 10)
}

function countedOnList(quotes: readonly Quote[]): QuoteTiles {
	const pending = quotes.filter((quote) => quote.status === 'SENT')
	const accepted = quotes.filter((quote) => quote.status === 'ACCEPTED')
	const declined = quotes.filter((quote) => quote.status === 'DECLINED')
	const decided = accepted.length + declined.length
	return {
		pending: tileOf(pending),
		accepted: tileOf(accepted),
		acceptanceRate:
			decided === 0 ? null : Math.round((accepted.length / decided) * 100),
		drafts: quotes.filter((quote) => quote.status === 'DRAFT').length,
	}
}

export function computeQuoteTiles(
	quotes: readonly Quote[],
	view?: QuoteTilesView,
): QuoteTiles {
	if (!view) return countedOnList(quotes)
	const since = addDays(view.today, -ACCEPTANCE_WINDOW_DAYS)
	const displayOf = (quote: Quote) =>
		quoteDisplayStatus(quote.status, view.extrasOf(quote.id), view.today)
	const answeredSince = (quote: Quote) =>
		answeredOn(quote, view.extrasOf(quote.id)) >= since

	const pending = quotes.filter((quote) => displayOf(quote) === 'sent')
	const accepted = quotes.filter(
		(quote) => quote.status === 'ACCEPTED' && answeredSince(quote),
	)
	const declined = quotes.filter(
		(quote) => quote.status === 'DECLINED' && answeredSince(quote),
	)
	const expired = quotes.filter(
		(quote) =>
			displayOf(quote) === 'expired' &&
			(view.extrasOf(quote.id).validUntil ?? '') >= since,
	)
	const decided = accepted.length + declined.length + expired.length
	return {
		pending: tileOf(pending),
		accepted: tileOf(accepted),
		acceptanceRate:
			decided === 0 ? null : Math.round((accepted.length / decided) * 100),
		drafts: quotes.filter((quote) => quote.status === 'DRAFT').length,
	}
}
