import { describe, expect, it } from 'vitest'
import {
	filterStripeEvents,
	parseStripeSearch,
	reviewActionOf,
	simulationNotice,
	stripeStats,
	stripeToReview,
	updateStripeSearch,
} from '#/pages/stripe/lib/stripe-model'
import type { StripeEvent } from '#/pages/stripe/types'

function event(overrides: Partial<StripeEvent>): StripeEvent {
	return {
		id: 'evt_1',
		type: 'payment_intent.succeeded',
		receivedAt: '2026-11-04T10:00:00.000Z',
		customerId: null,
		customerEmail: '',
		customerName: 'Lemoine',
		amount: 1000,
		description: '',
		stripeInvoiceNumber: null,
		paymentIntentId: null,
		mestierInvoiceNumber: null,
		lines: [],
		status: 'processed',
		reviewReason: null,
		accountId: null,
		accountName: '',
		paymentId: null,
		outcome: '',
		handledById: 'webhook',
		...overrides,
	}
}

describe('search', () => {
	it('keeps only known values and merges patches', () => {
		expect(
			parseStripeSearch({ statut: 'x', type: 'invoice.paid', q: ' a ' }),
		).toEqual({
			type: 'invoice.paid',
			q: 'a',
		})
		expect(
			updateStripeSearch({ q: 'a' }, { q: null, statut: 'ignored' }),
		).toEqual({
			statut: 'ignored',
		})
	})
})

describe('events', () => {
	const events = [
		event({ id: 'a', receivedAt: '2026-11-01T10:00:00.000Z' }),
		event({
			id: 'b',
			receivedAt: '2026-11-03T10:00:00.000Z',
			status: 'to_review',
			reviewReason: 'unknown_customer',
			customerName: 'Martel',
		}),
		event({
			id: 'c',
			receivedAt: '2026-08-01T10:00:00.000Z',
			status: 'ignored',
		}),
	]

	it('sorts the history newest first and filters it', () => {
		expect(filterStripeEvents(events, {}).map((item) => item.id)).toEqual([
			'b',
			'a',
			'c',
		])
		expect(
			filterStripeEvents(events, { statut: 'to_review' }).map(
				(item) => item.id,
			),
		).toEqual(['b'])
		expect(filterStripeEvents(events, { q: 'martel' })).toHaveLength(1)
	})

	it('counts the recent window and every pending review', () => {
		expect(stripeStats(events, '2026-11-05')).toEqual({
			received: 2,
			toReview: 1,
			automatic: 1,
			ignored: 0,
		})
		expect(stripeToReview(events).map((item) => item.id)).toEqual(['b'])
	})

	it('chooses the review action', () => {
		expect(reviewActionOf(events[1] as StripeEvent)).toBe('link')
		expect(
			reviewActionOf(
				event({ reviewReason: 'no_open_invoice', paymentId: 'p' }),
			),
		).toBe('allocate')
		expect(reviewActionOf(event({ reviewReason: 'no_open_invoice' }))).toBe(
			'none',
		)
	})

	it('words the simulation outcome', () => {
		expect(simulationNotice('invoice_paid', null)).toContain('aucun client')
		expect(
			simulationNotice(
				'payment_succeeded',
				event({ outcome: 'Paiement affecté à F-1' }),
			),
		).toBe('Webhook simulé, traité automatiquement : Paiement affecté à F-1.')
		expect(
			simulationNotice('unknown_customer', events[1] as StripeEvent),
		).toContain('Client Stripe inconnu')
	})
})
