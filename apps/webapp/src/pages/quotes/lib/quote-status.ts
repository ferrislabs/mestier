import type { QuoteStatus } from '#/hooks/use-quotes'
import type { QuoteExtras } from '#/pages/quotes/lib/quote-extras'

export type QuoteDisplayStatus =
	| 'draft'
	| 'sent'
	| 'accepted'
	| 'declined'
	| 'cancelled'
	| 'superseded'
	| 'expired'

export const QUOTE_DISPLAY_LABELS: Record<QuoteDisplayStatus, string> = {
	draft: 'Brouillon',
	sent: 'Envoyé',
	accepted: 'Accepté',
	declined: 'Refusé',
	cancelled: 'Annulé',
	superseded: 'Remplacé',
	expired: 'Expiré',
}

export function toIsoDate(date: Date): string {
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${date.getFullYear()}-${month}-${day}`
}

export function quoteDisplayStatus(
	status: QuoteStatus,
	extras: Pick<QuoteExtras, 'validUntil' | 'supersededBy'>,
	today: string,
): QuoteDisplayStatus {
	if (
		(status === 'SENT' || status === 'DECLINED') &&
		extras.supersededBy !== null
	)
		return 'superseded'
	if (
		status === 'SENT' &&
		extras.validUntil !== null &&
		extras.validUntil < today
	)
		return 'expired'
	switch (status) {
		case 'DRAFT':
			return 'draft'
		case 'SENT':
			return 'sent'
		case 'ACCEPTED':
			return 'accepted'
		case 'DECLINED':
			return 'declined'
		case 'CANCELLED':
			return 'cancelled'
	}
}
