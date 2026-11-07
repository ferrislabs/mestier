import type { Quote } from '#/hooks/use-quotes'

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
	quote: Pick<Quote, 'status' | 'is_expired'>,
): QuoteDisplayStatus {
	if (quote.is_expired) return 'expired'
	switch (quote.status) {
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
		case 'SUPERSEDED':
			return 'superseded'
	}
}
