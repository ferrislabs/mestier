import { describe, expect, it } from 'vitest'
import { EMPTY_QUOTE_EXTRAS } from './quote-extras'
import {
	QUOTE_DISPLAY_LABELS,
	quoteDisplayStatus,
	toIsoDate,
} from './quote-status'

const TODAY = '2026-11-07'

function extras(overrides: Partial<typeof EMPTY_QUOTE_EXTRAS> = {}) {
	return { ...EMPTY_QUOTE_EXTRAS, ...overrides }
}

describe('quoteDisplayStatus', () => {
	it('maps the backend statuses one to one', () => {
		expect(quoteDisplayStatus('DRAFT', extras(), TODAY)).toBe('draft')
		expect(quoteDisplayStatus('SENT', extras(), TODAY)).toBe('sent')
		expect(quoteDisplayStatus('ACCEPTED', extras(), TODAY)).toBe('accepted')
		expect(quoteDisplayStatus('DECLINED', extras(), TODAY)).toBe('declined')
		expect(quoteDisplayStatus('CANCELLED', extras(), TODAY)).toBe('cancelled')
	})

	it('reads a sent quote past its validity as expired', () => {
		expect(
			quoteDisplayStatus('SENT', extras({ validUntil: '2026-11-06' }), TODAY),
		).toBe('expired')
	})

	it('keeps a sent quote valid through the last day', () => {
		expect(
			quoteDisplayStatus('SENT', extras({ validUntil: TODAY }), TODAY),
		).toBe('sent')
	})

	it('never expires an accepted quote', () => {
		expect(
			quoteDisplayStatus(
				'ACCEPTED',
				extras({ validUntil: '2026-01-01' }),
				TODAY,
			),
		).toBe('accepted')
	})

	it('reads a sent or declined quote with a revision as replaced', () => {
		const replaced = extras({ supersededBy: { id: 'q2', label: 'DEV-002' } })

		expect(quoteDisplayStatus('SENT', replaced, TODAY)).toBe('superseded')
		expect(quoteDisplayStatus('DECLINED', replaced, TODAY)).toBe('superseded')
		expect(quoteDisplayStatus('DRAFT', replaced, TODAY)).toBe('draft')
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
