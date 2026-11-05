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
