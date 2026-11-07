import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import { connectorCopy } from '#/pages/automation/lib/connector-copy'
import { runNodeAppearance } from '#/pages/automation/lib/run-appearance'

function connector(kind: string): Schemas.PlacedConnectorDto {
	return { id: 'c1', kind, version: 1, config: {} }
}

describe('runNodeAppearance', () => {
	it('gives a connector the icon and hue of its kind, as the editor does', () => {
		const copy = connectorCopy('flow.condition')

		expect(
			runNodeAppearance({
				kind: 'connector',
				trigger: null,
				family: 'flow',
				connector: connector('flow.condition'),
			}),
		).toEqual({ icon: copy?.icon, hue: copy?.hue })
	})

	it('tells two kinds of the same family apart', () => {
		const assign = runNodeAppearance({
			kind: 'connector',
			trigger: null,
			family: 'mestier',
			connector: connector('mestier.support.assign_ticket'),
		})
		const badge = runNodeAppearance({
			kind: 'connector',
			trigger: null,
			family: 'mestier',
			connector: connector('mestier.support.add_badge'),
		})

		expect(assign).not.toEqual(badge)
	})

	it('falls back on the family for a kind it has no copy for', () => {
		expect(
			runNodeAppearance({
				kind: 'connector',
				trigger: null,
				family: 'http',
				connector: connector('http.unknown'),
			}),
		).toEqual(connectorAppearance('http'))
	})

	it('falls back on the default for a node the graph no longer knows', () => {
		expect(runNodeAppearance(undefined)).toEqual(connectorAppearance(undefined))
	})

	it('keeps the appearance of the manual and event triggers', () => {
		expect(
			runNodeAppearance({
				kind: 'trigger',
				trigger: { id: 't1', kind: 'Manual' },
				family: undefined,
				connector: null,
			}),
		).toEqual(triggerAppearance(true))
		expect(
			runNodeAppearance({
				kind: 'trigger',
				trigger: { id: 't1', kind: { Events: ['quote.accepted'] } },
				family: undefined,
				connector: null,
			}),
		).toEqual(triggerAppearance(false))
	})

	it('draws the webhook and the scheduler with the icon and hue of their own, not those of the event', () => {
		const webhook = runNodeAppearance({
			kind: 'trigger',
			trigger: {
				id: 't1',
				kind: { Webhook: { method: 'POST', path: 'paiement-recu' } },
			},
			family: undefined,
			connector: null,
		})
		const schedule = runNodeAppearance({
			kind: 'trigger',
			trigger: {
				id: 't2',
				kind: { Schedule: { frequency: 'Daily', time: '09:00' } },
			},
			family: undefined,
			connector: null,
		})

		expect(webhook).toEqual(triggerAppearance('webhook'))
		expect(webhook.hue).toBe(335)
		expect(schedule).toEqual(triggerAppearance('schedule'))
		expect(schedule.hue).toBe(255)
	})

	it('falls back on the event look for a trigger node whose trigger is gone', () => {
		expect(
			runNodeAppearance({
				kind: 'trigger',
				trigger: null,
				family: undefined,
				connector: null,
			}),
		).toEqual(triggerAppearance(false))
	})
})
