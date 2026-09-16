import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { groupEventsByPrefix } from '#/pages/automation/lib/trigger-events'

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
