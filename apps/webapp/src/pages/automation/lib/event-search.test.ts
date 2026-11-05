import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { searchEvents } from '#/pages/automation/lib/event-search'

function event(name: string, label: string): Schemas.EventDescriptorResponse {
	return {
		name,
		label,
		subject_kind: 'quote',
		version: 1,
		payload_example: {},
	}
}

const events = [
	event('quote.sent', 'Devis envoyé'),
	event('invoice.paid', 'Facture payée'),
]

describe('searchEvents', () => {
	it('groups by domain with a French label', () => {
		expect(searchEvents(events, '').map((group) => group.label)).toEqual([
			'Factures',
			'Devis',
		])
	})

	it('matches the label, the name or the domain, ignoring accents', () => {
		expect(searchEvents(events, 'envoye')[0].events[0].name).toBe('quote.sent')
		expect(searchEvents(events, 'invoice.paid')).toHaveLength(1)
		expect(searchEvents(events, 'factures')[0].prefix).toBe('invoice')
	})

	it('returns nothing when nothing matches', () => {
		expect(searchEvents(events, 'zzz')).toEqual([])
	})
})
