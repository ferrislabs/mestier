import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { triggerPresentation } from '#/pages/automation/lib/trigger-events'

function event(name: string): Schemas.EventDescriptorResponse {
	return {
		name,
		label: name,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: {},
	}
}

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
