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

	it('keeps the family look for a kind it does not know', () => {
		expect(connectorAppearance('http', 'http.unknown')).toEqual(
			connectorAppearance('http'),
		)
	})

	it('draws the triggers from the trigger copy', () => {
		expect(triggerAppearance(true).hue).toBe(triggerCopy('manual').hue)
		expect(triggerAppearance(false).hue).toBe(triggerCopy('events').hue)
	})
})
