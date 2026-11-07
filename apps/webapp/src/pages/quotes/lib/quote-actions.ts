import type { QuoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

export type QuoteAction =
	| 'send'
	| 'accept'
	| 'decline'
	| 'cancel'
	| 'reopen'
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
			return [
				'accept',
				'decline',
				'revise',
				'cancel',
				'reopen',
				'duplicate',
				'delete',
			]
		case 'expired':
			return ['revise', 'cancel', 'reopen', 'duplicate', 'delete']
		case 'accepted':
			return ['project', 'invoice', 'duplicate', 'delete']
		case 'declined':
			return ['revise', 'reopen', 'duplicate', 'delete']
		case 'cancelled':
			return ['reopen', 'duplicate', 'delete']
		case 'superseded':
			return ['duplicate', 'delete']
	}
}
