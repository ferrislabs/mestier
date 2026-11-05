import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorNode,
	type EditorLookup,
	findOpenOutput,
	toEditorEdges,
	toEditorNodes,
	toGraph,
	toMeta,
	triggerNode,
} from '#/pages/automation/lib/editor-nodes'

const descriptors = new Map<string, Schemas.ConnectorDescriptorResponse>([
	[
		'flow.if',
		{
			kind: 'flow.if',
			label: 'Condition',
			family: 'flow',
			version: 1,
			auth: 'None',
			fields: [],
			branches: ['Then', 'Else'],
			output_example: null,
		},
	],
])
const lookup: EditorLookup = { descriptors, events: [] }

const graph: Schemas.GraphDto = {
	triggers: [{ id: 't1', kind: 'Manual' }],
	connectors: [{ id: 'c1', kind: 'flow.if', version: 1, config: { a: 1 } }],
	edges: [
		{ from: 't1', to: 'c1', branch: null },
		{ from: 'c1', to: 'c1', branch: 'Then' },
	],
}

describe('editor nodes', () => {
	it('names and describes a known connector after the copy table', () => {
		const known = new Map<string, Schemas.ConnectorDescriptorResponse>([
			[
				'flow.condition',
				{
					kind: 'flow.condition',
					label: 'Condition',
					family: 'flow',
					version: 1,
					auth: 'None',
					fields: [],
					branches: ['Then', 'Else'],
					output_example: null,
				},
			],
			[
				'mestier.support.add_badge',
				{
					kind: 'mestier.support.add_badge',
					label: 'Ajouter un badge',
					family: 'mestier',
					version: 1,
					auth: 'None',
					fields: [],
					branches: [],
					output_example: null,
				},
			],
		])
		const lookupKnown: EditorLookup = { descriptors: known, events: [] }

		const condition = connectorNode(
			{ id: 'c1', kind: 'flow.condition', version: 1, config: {} },
			{ x: 0, y: 0 },
			lookupKnown,
		)
		const badge = connectorNode(
			{ id: 'c2', kind: 'mestier.support.add_badge', version: 1, config: {} },
			{ x: 0, y: 0 },
			lookupKnown,
		)

		expect(condition.data).toMatchObject({
			label: 'Si',
			subtitle: 'Condition',
			typeLabel: 'Si',
			family: 'flow',
		})
		expect(badge.data).toMatchObject({
			label: 'Ajouter un badge au ticket',
			subtitle: 'Action Mestier',
		})
	})

	it('describes an unknown connector with the descriptor label and a generic subtitle', () => {
		const node = connectorNode(graph.connectors[0], { x: 0, y: 0 }, lookup)
		expect(node.data).toMatchObject({
			label: 'Condition',
			subtitle: 'Connecteur',
			typeLabel: 'Condition',
		})
	})

	it('types a trigger after its kind rather than a generic word', () => {
		const manual = triggerNode(graph.triggers[0], { x: 0, y: 0 }, lookup)
		const events = triggerNode(
			{ id: 't2', kind: { Events: ['quote.sent'] } },
			{ x: 0, y: 0 },
			lookup,
		)
		expect(manual.data.typeLabel).toBe('Clic')
		expect(events.data.typeLabel).toBe('Event Mestier')
	})

	it('names a connector after its descriptor and a trigger after its kind', () => {
		const nodes = toEditorNodes(
			graph,
			new Map(),
			lookup,
			{},
			new Map(),
			new Map(),
		)
		expect(nodes.map((node) => node.data.label)).toEqual(['Clic', 'Condition'])
		expect(nodes[1].data.branches).toEqual(['Then', 'Else'])
	})

	it('lets a stored name, description and flag override the defaults', () => {
		const node = connectorNode(graph.connectors[0], { x: 0, y: 0 }, lookup, {
			label: 'Mon test',
			subtitle: 'Pourquoi',
			disabled: true,
		})
		expect(node.data).toMatchObject({
			label: 'Mon test',
			subtitle: 'Pourquoi',
			disabled: true,
		})
	})

	it('round-trips the graph through nodes and edges', () => {
		const nodes = toEditorNodes(
			graph,
			new Map(),
			lookup,
			{},
			new Map(),
			new Map(),
		)
		expect(toGraph(nodes, toEditorEdges(graph))).toEqual(graph)
	})

	it('keeps only the non-default meta', () => {
		const plain = triggerNode(graph.triggers[0], { x: 0, y: 0 }, lookup)
		const named = triggerNode(graph.triggers[0], { x: 0, y: 0 }, lookup, {
			label: 'Départ',
		})
		expect(toMeta([plain])).toEqual({})
		expect(toMeta([{ ...named, id: 't2' }])).toEqual({
			t2: { label: 'Départ' },
		})
	})

	it('finds the first output that has no edge yet', () => {
		const node = connectorNode(graph.connectors[0], { x: 0, y: 0 }, lookup)
		expect(
			findOpenOutput(node, [
				{ id: 'e', source: 'c1', target: 'x', sourceHandle: 'Then' },
			]),
		).toEqual({ nodeId: 'c1', outputId: 'Else' })
		expect(
			findOpenOutput(node, [
				{ id: 'e', source: 'c1', target: 'x', sourceHandle: 'Then' },
				{ id: 'f', source: 'c1', target: 'y', sourceHandle: 'Else' },
			]),
		).toBeNull()
	})
})
