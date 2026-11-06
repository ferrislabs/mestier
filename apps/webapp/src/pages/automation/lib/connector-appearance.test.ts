import { CalendarClock, Library, Webhook } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import {
	connectorAppearance,
	TRIGGER_APPEARANCE,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import {
	connectorCopy,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'

describe('connectorAppearance', () => {
	it('gives each known family its own hue', () => {
		const hues = ['mestier', 'http', 'odoo', 'flow'].map(
			(family) => connectorAppearance(family).hue,
		)
		expect(new Set(hues).size).toBe(4)
	})

	it('falls back for an unknown or missing family', () => {
		expect(connectorAppearance('unknown')).toEqual(
			connectorAppearance(undefined),
		)
		expect(connectorAppearance('unknown')).not.toEqual(TRIGGER_APPEARANCE)
	})

	it('tells a manual trigger from an event trigger', () => {
		expect(triggerAppearance(true)).not.toEqual(triggerAppearance(false))
	})

	it('takes the icon and hue of the kind when it is known, whatever its family', () => {
		const assign = connectorCopy('mestier.support.assign_ticket')
		const badge = connectorCopy('mestier.support.add_badge')
		expect(
			connectorAppearance('mestier', 'mestier.support.assign_ticket'),
		).toEqual({ icon: assign?.icon, hue: assign?.hue })
		expect(connectorAppearance('mestier', 'mestier.support.add_badge')).toEqual(
			{ icon: badge?.icon, hue: badge?.hue },
		)
		expect(
			connectorAppearance('mestier', 'mestier.support.assign_ticket'),
		).not.toEqual(connectorAppearance('mestier', 'mestier.support.add_badge'))
	})

	it('draws a collection operation as a library at hue 200, not as a generic http call', () => {
		expect(connectorAppearance('http', 'collection.operation')).toEqual({
			icon: Library,
			hue: 200,
		})
		expect(connectorAppearance('http', 'collection.operation')).not.toEqual(
			connectorAppearance('http', 'http.request'),
		)
	})

	it('keeps the family look for a kind it does not know', () => {
		expect(connectorAppearance('http', 'http.unknown')).toEqual(
			connectorAppearance('http'),
		)
	})

	it('draws the triggers from the trigger copy', () => {
		expect(triggerAppearance(true).hue).toBe(triggerCopy('manual').hue)
		expect(triggerAppearance(false).hue).toBe(triggerCopy('events').hue)
	})

	it('draws the scheduler and the webhook with the icon and hue of the prototype', () => {
		expect(triggerAppearance('schedule')).toEqual({
			icon: CalendarClock,
			hue: 255,
		})
		expect(triggerAppearance('webhook')).toEqual({ icon: Webhook, hue: 335 })
	})

	it('tells the four triggers apart', () => {
		const hues = (['schedule', 'events', 'webhook', 'manual'] as const).map(
			(kind) => triggerAppearance(kind).hue,
		)
		expect(new Set(hues).size).toBe(4)
	})
})
