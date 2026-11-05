import type { QuoteStatus } from '#/hooks/use-quotes'

export type QuoteAction =
	| 'send'
	| 'accept'
	| 'decline'
	| 'cancel'
	| 'reopen'
	| 'project'
	| 'duplicate'
	| 'delete'

export function availableQuoteActions(status: QuoteStatus): QuoteAction[] {
	switch (status) {
		case 'DRAFT':
			return ['send', 'duplicate', 'delete']
		case 'SENT':
			return ['accept', 'decline', 'cancel', 'reopen', 'duplicate', 'delete']
		case 'ACCEPTED':
			return ['project', 'duplicate', 'delete']
		case 'DECLINED':
		case 'CANCELLED':
			return ['reopen', 'duplicate', 'delete']
	}
}
