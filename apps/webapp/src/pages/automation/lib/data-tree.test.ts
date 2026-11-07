import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	resolveTriggerExample,
	toEvaluateContext,
} from '#/pages/automation/lib/data-tree'

describe('toEvaluateContext', () => {
	it('gives the items of each node under its name, at the first position', () => {
		const context = toEvaluateContext({
			trigger: { a: 1 },
			nodes: [{ label: 'Tarifs', items: [{ id: 42 }, { id: 43 }] }],
		})

		expect(context).toEqual({
			trigger: { a: 1 },
			nodes: { Tarifs: [{ id: 42 }, { id: 43 }] },
			position: 0,
			loop: null,
		})
	})

	it('passes an empty node list through as an empty object', () => {
		const context = toEvaluateContext({ trigger: null, nodes: [] })
		expect(context.nodes).toEqual({})
	})

	it('reads the item at the position asked for', () => {
		expect(toEvaluateContext({ trigger: null, nodes: [] }, 2).position).toBe(2)
	})

	it('carries the pass of the loop the node is in', () => {
		expect(
			toEvaluateContext({ trigger: null, nodes: [] }, 0, {
				index: 1,
				item: 'b',
			}).loop,
		).toEqual({ index: 1, item: 'b' })
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
