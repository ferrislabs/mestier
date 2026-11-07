import type { QuoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

export type QuoteAction =
	| 'send'
	| 'accept'
	| 'decline'
	| 'cancel'
	| 'revise'
	| 'project'
	| 'invoice'
	| 'duplicate'
	| 'delete'

export function availableQuoteActions(
	status: QuoteDisplayStatus,
): QuoteAction[] {
	switch (status) {
		case 'draft':
			return ['send', 'duplicate', 'delete']
		case 'sent':
			return ['accept', 'decline', 'revise', 'cancel', 'duplicate']
		case 'expired':
			return ['revise', 'cancel', 'duplicate']
		case 'accepted':
			return ['project', 'invoice', 'duplicate']
		case 'declined':
			return ['revise', 'duplicate']
		case 'cancelled':
		case 'superseded':
			return ['duplicate']
	}
}
