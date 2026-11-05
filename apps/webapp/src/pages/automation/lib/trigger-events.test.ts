import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	groupEventsByPrefix,
	triggerPresentation,
} from '#/pages/automation/lib/trigger-events'

function event(name: string): Schemas.EventDescriptorResponse {
	return {
		name,
		label: name,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: {},
	}
}

describe('groupEventsByPrefix', () => {
	it('groups events by the segment before the first dot', () => {
		const groups = groupEventsByPrefix([
			event('quote.accepted'),
			event('invoice.paid'),
			event('quote.sent'),
		])

		expect(groups).toEqual([
			{ prefix: 'invoice', events: [event('invoice.paid')] },
			{
				prefix: 'quote',
				events: [event('quote.accepted'), event('quote.sent')],
			},
		])
	})

	it('keeps a multi-word prefix whole', () => {
		const groups = groupEventsByPrefix([event('time_entry.stopped')])

		expect(groups).toEqual([
			{ prefix: 'time_entry', events: [event('time_entry.stopped')] },
		])
	})

	it('sorts groups alphabetically for a stable listing', () => {
		const groups = groupEventsByPrefix([
			event('supplier_invoice.paid'),
			event('customer.created'),
		])

		expect(groups.map((g) => g.prefix)).toEqual([
			'customer',
			'supplier_invoice',
		])
	})

	it('is empty for an empty catalogue', () => {
		expect(groupEventsByPrefix([])).toEqual([])
	})
})

describe('triggerPresentation', () => {
	const events = [event('quote.sent')]

	it('names a manual trigger', () => {
		expect(triggerPresentation({ id: 't1', kind: 'Manual' }, events)).toEqual({
			label: 'Clic',
			subtitle: 'Déclenchement manuel',
		})
	})

	it('asks to choose the event when none is selected', () => {
		expect(
			triggerPresentation({ id: 't1', kind: { Events: [] } }, events),
		).toEqual({
			label: 'Event Mestier',
			subtitle: 'Cliquez pour choisir l’événement',
		})
	})

	it('names a single event after its label', () => {
		expect(
			triggerPresentation(
				{ id: 't1', kind: { Events: ['quote.sent'] } },
				events,
			),
		).toEqual({ label: 'quote.sent', subtitle: 'Event Mestier · quote.sent' })
	})

	it('counts several events', () => {
		expect(
			triggerPresentation(
				{ id: 't1', kind: { Events: ['quote.sent', 'quote.signed'] } },
				events,
			).subtitle,
		).toBe('2 événements')
	})
})
