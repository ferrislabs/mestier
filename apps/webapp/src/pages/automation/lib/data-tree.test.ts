import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildAvailableDataTree,
	resolveTriggerExample,
	toEvaluateContext,
} from '#/pages/automation/lib/data-tree'

function leaf(node: ReturnType<typeof buildAvailableDataTree>[number]) {
	return node
}

describe('buildAvailableDataTree', () => {
	it('builds a trigger branch whose leaves carry the trigger. path prefix', () => {
		const tree = buildAvailableDataTree({
			trigger: { customer: { id: 'cus_1', name: 'Julie' } },
			connectors: [],
		})

		const trigger = tree.find((node) => node.key === 'trigger')
		expect(trigger?.kind).toBe('branch')
		if (trigger?.kind !== 'branch') throw new Error('expected a branch')

		const customer = trigger.children.find((node) => node.key === 'customer')
		expect(customer?.kind).toBe('branch')
		if (customer?.kind !== 'branch') throw new Error('expected a branch')
		expect(customer.path).toBe('trigger.customer')

		const id = customer.children.find((node) => node.key === 'id')
		expect(id).toEqual({
			kind: 'leaf',
			key: 'id',
			label: 'id',
			path: 'trigger.customer.id',
			value: 'cus_1',
		})
	})

	it('indexes an array with bracket segments rather than dotted ones', () => {
		const tree = buildAvailableDataTree({
			trigger: { lines: [{ total: 12 }, { total: 34 }] },
			connectors: [],
		})

		const trigger = tree.find((node) => node.key === 'trigger')
		if (trigger?.kind !== 'branch') throw new Error('expected a branch')
		const lines = trigger.children.find((node) => node.key === 'lines')
		if (lines?.kind !== 'branch') throw new Error('expected a branch')
		const second = lines.children[1]
		if (second?.kind !== 'branch') throw new Error('expected a branch')
		expect(second.path).toBe('trigger.lines[1]')

		const total = second.children.find((node) => node.key === 'total')
		expect(total).toMatchObject({
			path: 'trigger.lines[1].total',
			value: 34,
		})
	})

	it('adds one branch per upstream connector, rooted at connectors.<id>.output', () => {
		const tree = buildAvailableDataTree({
			trigger: null,
			connectors: [
				{ id: 'c1', label: 'Créer un client', output: { id: 42 } },
				{ id: 'c2', label: 'Requête HTTP', output: { status: 200 } },
			],
		})

		const ids = tree.filter((node) => node.key !== 'trigger').map((n) => n.path)
		expect(ids).toEqual(['connectors.c1.output', 'connectors.c2.output'])

		const c1 = tree.find((node) => node.path === 'connectors.c1.output')
		if (c1?.kind !== 'branch') throw new Error('expected a branch')
		expect(leaf(c1.children[0])).toEqual({
			kind: 'leaf',
			key: 'id',
			label: 'id',
			path: 'connectors.c1.output.id',
			value: 42,
		})
	})

	it('leaves a scalar trigger payload with no children', () => {
		const tree = buildAvailableDataTree({ trigger: 'raw', connectors: [] })
		const trigger = tree.find((node) => node.key === 'trigger')
		if (trigger?.kind !== 'branch') throw new Error('expected a branch')
		expect(trigger.children).toEqual([])
	})

	it('replaces the trigger branch with a notice when no trigger event is configured', () => {
		const tree = buildAvailableDataTree({ trigger: undefined, connectors: [] })
		const trigger = tree.find((node) => node.key === 'trigger')
		expect(trigger?.kind).toBe('notice')
		if (trigger?.kind !== 'notice') throw new Error('expected a notice')
		expect(trigger.message.length).toBeGreaterThan(0)
	})

	it('replaces the trigger branch with the same notice when the trigger payload is null', () => {
		const tree = buildAvailableDataTree({ trigger: null, connectors: [] })
		const trigger = tree.find((node) => node.key === 'trigger')
		expect(trigger?.kind).toBe('notice')
	})

	it('keeps upstream connector branches untouched when the trigger has no payload', () => {
		const tree = buildAvailableDataTree({
			trigger: undefined,
			connectors: [{ id: 'c1', label: 'Créer un client', output: { id: 42 } }],
		})
		const connector = tree.find((node) => node.path === 'connectors.c1.output')
		expect(connector?.kind).toBe('branch')
		if (connector?.kind !== 'branch') throw new Error('expected a branch')
		expect(connector.children).toEqual([
			{
				kind: 'leaf',
				key: 'id',
				label: 'id',
				path: 'connectors.c1.output.id',
				value: 42,
			},
		])
	})
})

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

	it('returns the payload example of the first selected event, in catalogue order', () => {
		expect(
			resolveTriggerExample(events, ['invoice.paid', 'quote.accepted']),
		).toEqual({ quote_id: 'q1' })
	})

	it('is undefined when nothing is selected', () => {
		expect(resolveTriggerExample(events, [])).toBeUndefined()
	})

	it('is undefined when the selection names an event outside the catalogue', () => {
		expect(resolveTriggerExample(events, ['unknown.event'])).toBeUndefined()
	})
})
