// MOCK, pas d'endpoint, remplacé au lot B3
import type { DealQuote } from '#/pages/deals/lib/deal-types'

const NO_QUOTES: readonly DealQuote[] = []

export function useDealQuotes(): readonly DealQuote[] {
	return NO_QUOTES
}
