import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	isEventsTrigger,
	isManualTrigger,
	scheduleOf,
	triggerEventNames,
	triggerExample,
	triggerKindOf,
	triggerPresentation,
	webhookOf,
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

const WEBHOOK: Schemas.TriggerKindDto = {
	Webhook: {
		method: 'POST',
		path: 'paiement-recu',
		credential_id: null,
		contract: [
			{ name: 'facture', kind: 'Text', required: true, example: 'FA-1' },
			{ name: 'montant', kind: 'Number', required: true, example: '2340' },
		],
		on_invalid: 'Reject',
	},
}

const SCHEDULE: Schemas.TriggerKindDto = {
	Schedule: { frequency: 'Daily', time: '09:00', timezone: 'Europe/Paris' },
}

describe('the kind of a trigger', () => {
	it('tells the four kinds apart', () => {
		expect(triggerKindOf('Manual')).toBe('manual')
		expect(triggerKindOf({ Events: [] })).toBe('events')
		expect(triggerKindOf(WEBHOOK)).toBe('webhook')
		expect(triggerKindOf(SCHEDULE)).toBe('schedule')
	})

	it('narrows a trigger to its own variant and to nothing else', () => {
		const webhook = { id: 't1', kind: WEBHOOK }
		const schedule = { id: 't2', kind: SCHEDULE }
		const events = { id: 't3', kind: { Events: ['quote.sent'] } }
		const manual = { id: 't4', kind: 'Manual' as const }

		expect(webhookOf(webhook)?.path).toBe('paiement-recu')
		expect(webhookOf(schedule)).toBeNull()
		expect(webhookOf(manual)).toBeNull()
		expect(scheduleOf(schedule)?.time).toBe('09:00')
		expect(scheduleOf(webhook)).toBeNull()
		expect(triggerEventNames(events)).toEqual(['quote.sent'])
		expect(triggerEventNames(webhook)).toEqual([])
		expect(triggerEventNames(schedule)).toEqual([])
		expect(triggerEventNames(manual)).toEqual([])
		expect(isEventsTrigger(events)).toBe(true)
		expect(isEventsTrigger(webhook)).toBe(false)
		expect(isManualTrigger(manual)).toBe(true)
		expect(isManualTrigger(schedule)).toBe(false)
	})
})

describe('triggerPresentation — the scheduler and the webhook', () => {
	it('names a scheduler « Planificateur » and reads its rule in the subtitle', () => {
		expect(triggerPresentation({ id: 't1', kind: SCHEDULE }, [])).toEqual({
			label: 'Planificateur',
			subtitle: 'tous les jours à 9:00',
		})
		expect(
			triggerPresentation(
				{
					id: 't1',
					kind: {
						Schedule: { frequency: 'Weekly', time: '14:30', weekday: 'Friday' },
					},
				},
				[],
			).subtitle,
		).toBe('toutes les semaines le vendredi à 14:30')
	})

	it('names a webhook « Webhook », the HTTP trigger of the prototype', () => {
		expect(triggerPresentation({ id: 't1', kind: WEBHOOK }, [])).toEqual({
			label: 'Webhook',
			subtitle: 'Déclencheur HTTP',
		})
	})
})

describe('triggerExample', () => {
	it('gives a webhook the body built from the examples of its contract', () => {
		expect(triggerExample({ id: 't1', kind: WEBHOOK }, [])).toEqual({
			facture: 'FA-1',
			montant: 2340,
		})
	})

	it('gives a scheduler the four keys of its payload', () => {
		expect(
			triggerExample(
				{ id: 't1', kind: SCHEDULE },
				[],
				new Date('2026-11-04T10:00:00Z'),
			),
		).toEqual({
			scheduled_for: '2026-11-05T08:00:00Z',
			horodatage: '2026-11-05T09:00:00+01:00',
			jour: 'jeudi',
			fuseau: 'Europe/Paris',
		})
	})

	it('gives an event trigger the payload of its first known event', () => {
		expect(
			triggerExample({ id: 't1', kind: { Events: ['x.y', 'quote.sent'] } }, [
				event('quote.sent'),
			]),
		).toEqual({})
	})

	it('has nothing for a click or an event that is not chosen', () => {
		expect(triggerExample({ id: 't1', kind: 'Manual' }, [])).toBeUndefined()
		expect(
			triggerExample({ id: 't1', kind: { Events: [] } }, []),
		).toBeUndefined()
	})
})
