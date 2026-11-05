import type { Quote } from '#/hooks/use-quotes'

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

function tileOf(quotes: readonly Quote[]): QuoteTile {
	return {
		count: quotes.length,
		netCents: quotes.reduce((sum, quote) => sum + quote.net_cents, 0),
	}
}

export function computeQuoteTiles(quotes: readonly Quote[]): QuoteTiles {
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
