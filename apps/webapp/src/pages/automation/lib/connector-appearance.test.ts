import { describe, expect, it } from 'vitest'
import {
	connectorAppearance,
	TRIGGER_APPEARANCE,
} from '#/pages/automation/lib/connector-appearance'

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
})
