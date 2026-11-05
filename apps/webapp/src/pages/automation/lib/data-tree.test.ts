import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	resolveTriggerExample,
	toEvaluateContext,
} from '#/pages/automation/lib/data-tree'

describe('toEvaluateContext', () => {
	it('wraps each connector output under an "output" key, matching the grammar', () => {
		const context = toEvaluateContext({
			trigger: { a: 1 },
			connectors: [{ id: 'c1', label: 'x', output: { id: 42 } }],
		})

		expect(context).toEqual({
			trigger: { a: 1 },
			connectors: { c1: { output: { id: 42 } } },
			loop: null,
		})
	})

	it('passes an empty connector list through as an empty object', () => {
		const context = toEvaluateContext({ trigger: null, connectors: [] })
		expect(context.connectors).toEqual({})
	})
})

describe('resolveTriggerExample', () => {
	const events: Schemas.EventDescriptorResponse[] = [
		{
			name: 'quote.accepted',
			label: 'Devis accepté',
			subject_kind: 'quote',
			version: 1,
			payload_example: { quote_id: 'q1' },
		},
		{
			name: 'invoice.paid',
			label: 'Facture payée',
			subject_kind: 'invoice',
			version: 1,
			payload_example: { invoice_id: 'i1' },
		},
	]

	it('returns the payload example of the first selected event, in selection order', () => {
		expect(
			resolveTriggerExample(events, ['invoice.paid', 'quote.accepted']),
		).toEqual({ invoice_id: 'i1' })
	})

	it('does not depend on the order the catalogue came in', () => {
		expect(
			resolveTriggerExample([...events].reverse(), [
				'quote.accepted',
				'invoice.paid',
			]),
		).toEqual({ quote_id: 'q1' })
	})

	it('skips a selected event the catalogue does not know', () => {
		expect(
			resolveTriggerExample(events, ['unknown.event', 'invoice.paid']),
		).toEqual({ invoice_id: 'i1' })
	})

	it('is undefined when nothing is selected', () => {
		expect(resolveTriggerExample(events, [])).toBeUndefined()
	})

	it('is undefined when the selection names an event outside the catalogue', () => {
		expect(resolveTriggerExample(events, ['unknown.event'])).toBeUndefined()
	})
})
