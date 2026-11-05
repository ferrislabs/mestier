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
})
