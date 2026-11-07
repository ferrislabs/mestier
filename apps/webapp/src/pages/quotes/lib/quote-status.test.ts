import { describe, expect, it } from 'vitest'
import {
	QUOTE_DISPLAY_LABELS,
	quoteDisplayStatus,
	toIsoDate,
} from './quote-status'

describe('quoteDisplayStatus', () => {
	it('maps the backend statuses one to one', () => {
		const show = (status: Parameters<typeof quoteDisplayStatus>[0]['status']) =>
			quoteDisplayStatus({ status, is_expired: false })

		expect(show('DRAFT')).toBe('draft')
		expect(show('SENT')).toBe('sent')
		expect(show('ACCEPTED')).toBe('accepted')
		expect(show('DECLINED')).toBe('declined')
		expect(show('CANCELLED')).toBe('cancelled')
		expect(show('SUPERSEDED')).toBe('superseded')
	})

	it('reads the expiry the backend computed, never a date of the browser', () => {
		expect(quoteDisplayStatus({ status: 'SENT', is_expired: true })).toBe(
			'expired',
		)
		expect(quoteDisplayStatus({ status: 'SENT', is_expired: false })).toBe(
			'sent',
		)
	})
})

describe('labels', () => {
	it('names every display status in French', () => {
		expect(QUOTE_DISPLAY_LABELS).toEqual({
			draft: 'Brouillon',
			sent: 'Envoyé',
			accepted: 'Accepté',
			declined: 'Refusé',
			cancelled: 'Annulé',
			superseded: 'Remplacé',
			expired: 'Expiré',
		})
	})
})

describe('toIsoDate', () => {
	it('keeps the local calendar day', () => {
		expect(toIsoDate(new Date(2026, 10, 7, 23, 30))).toBe('2026-11-07')
	})
})
