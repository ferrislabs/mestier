import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorNode,
	type EditorLookup,
	findOpenOutput,
	toEditorEdges,
	toEditorNodes,
	toGraph,
	triggerNode,
	withDetails,
	withDisabled,
	withRenamedReferences,
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
			execution: 'per_item',
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
					execution: 'per_item',
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
					execution: 'per_item',
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
		const nodes = toEditorNodes(graph, new Map(), lookup, new Map(), new Map())
		expect(nodes.map((node) => node.data.label)).toEqual(['Clic', 'Condition'])
		expect(nodes[1].data.branches).toEqual(['Then', 'Else'])
	})

	it('lets the name, description and flag saved with the node override the defaults', () => {
		const node = connectorNode(
			{
				...graph.connectors[0],
				name: 'Mon test',
				description: 'Pourquoi',
				disabled: true,
			},
			{ x: 0, y: 0 },
			lookup,
		)
		expect(node.data).toMatchObject({
			label: 'Mon test',
			subtitle: 'Pourquoi',
			disabled: true,
		})
	})

	it('falls back to the default name when the saved one is blank, and keeps an emptied description empty', () => {
		const node = connectorNode(
			{ ...graph.connectors[0], name: '', description: '' },
			{ x: 0, y: 0 },
			lookup,
		)
		expect(node.data).toMatchObject({ label: 'Condition', subtitle: '' })
	})

	it('names and describes a trigger from the graph and never disables it', () => {
		const node = triggerNode(
			{ ...graph.triggers[0], name: 'Départ', description: 'Au clic' },
			{ x: 0, y: 0 },
			lookup,
		)
		expect(node.data).toMatchObject({
			label: 'Départ',
			subtitle: 'Au clic',
			disabled: false,
		})
	})

	it('round-trips the graph through nodes and edges', () => {
		const nodes = toEditorNodes(graph, new Map(), lookup, new Map(), new Map())
		expect(toGraph(nodes, toEditorEdges(graph))).toEqual(graph)
	})

	it('carries the operation a node calls in the graph, with no copy of it on the side', () => {
		const operation = {
			collection_id: '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11',
			operation_id: 'list',
		}
		const withOperation: Schemas.GraphDto = {
			...graph,
			connectors: [
				{
					id: 'c1',
					kind: 'collection.operation',
					version: 1,
					config: { query: { limit: 5 } },
					operation,
				},
			],
			edges: [],
		}

		const nodes = toEditorNodes(
			withOperation,
			new Map(),
			lookup,
			new Map(),
			new Map(),
		)

		expect(nodes[1]?.data.connector?.operation).toEqual(operation)
		expect(toGraph(nodes, []).connectors).toEqual(withOperation.connectors)
	})

	it('names and describes a collection operation after its copy, until its own name is set', () => {
		const node = connectorNode(
			{
				id: 'c1',
				kind: 'collection.operation',
				version: 1,
				config: {},
				operation: { collection_id: 'x', operation_id: 'y' },
			},
			{ x: 0, y: 0 },
			lookup,
		)

		expect(node.data).toMatchObject({
			label: 'Opération de collection',
			subtitle: 'Collection',
			typeLabel: 'Opération de collection',
		})
	})

	it('writes a new name and description on the connector or trigger itself', () => {
		const connector = connectorNode(graph.connectors[0], { x: 0, y: 0 }, lookup)
		const trigger = triggerNode(graph.triggers[0], { x: 0, y: 0 }, lookup)

		const renamed = withDetails(connector, { name: 'Vérifier' }, lookup)
		const described = withDetails(trigger, { description: 'Au clic' }, lookup)

		expect(renamed.data.connector?.name).toBe('Vérifier')
		expect(renamed.data.label).toBe('Vérifier')
		expect(described.data.trigger?.description).toBe('Au clic')
		expect(described.data.subtitle).toBe('Au clic')
		expect(toGraph([renamed], []).connectors[0]).toMatchObject({
			id: 'c1',
			name: 'Vérifier',
		})
	})

	it('disables a connector in the graph and leaves a trigger alone', () => {
		const connector = connectorNode(graph.connectors[0], { x: 0, y: 0 }, lookup)
		const trigger = triggerNode(graph.triggers[0], { x: 0, y: 0 }, lookup)

		const off = withDisabled(connector, true, lookup)

		expect(off.data.connector?.disabled).toBe(true)
		expect(off.data.disabled).toBe(true)
		expect(withDisabled(off, false, lookup).data.disabled).toBe(false)
		expect(withDisabled(trigger, true, lookup)).toBe(trigger)
	})

	it('rewrites the references to a renamed node in every other node, nested ones included', () => {
		const renamed = connectorNode(
			{ id: 'c1', kind: 'flow.if', version: 1, config: {}, name: 'Tarifs' },
			{ x: 0, y: 0 },
			lookup,
		)
		const reader = connectorNode(
			{
				id: 'c2',
				kind: 'http.request',
				version: 1,
				config: {
					url: "{{ $('Tarifs').item.json.url }}",
					body: { lignes: [{ code: "{{ $('Tarifs').item.json.code }}" }] },
				},
			},
			{ x: 0, y: 0 },
			lookup,
		)
		const other = connectorNode(
			{ id: 'c3', kind: 'http.request', version: 1, config: { a: 'x' } },
			{ x: 0, y: 0 },
			lookup,
		)

		const next = withRenamedReferences(
			[renamed, reader, other],
			'c1',
			'Tarifs',
			'Prix fournisseur',
		)

		expect(next[1].data.connector?.config).toEqual({
			url: "{{ $('Prix fournisseur').item.json.url }}",
			body: {
				lignes: [{ code: "{{ $('Prix fournisseur').item.json.code }}" }],
			},
		})
		expect(next[0]).toBe(renamed)
		expect(next[2]).toBe(other)
	})

	it('leaves the config of the renamed node itself alone', () => {
		const self = connectorNode(
			{
				id: 'c1',
				kind: 'flow.if',
				version: 1,
				config: { a: "{{ $('Tarifs').item.json.x }}" },
				name: 'Tarifs',
			},
			{ x: 0, y: 0 },
			lookup,
		)

		expect(withRenamedReferences([self], 'c1', 'Tarifs', 'Autre')[0]).toBe(self)
	})

	it('saves under its name a node that is read by another one but was never named', () => {
		const source = connectorNode(
			{ id: 'c1', kind: 'flow.if', version: 1, config: {} },
			{ x: 0, y: 0 },
			lookup,
		)
		const reader = connectorNode(
			{
				id: 'c2',
				kind: 'http.request',
				version: 1,
				config: { url: "{{ $('Condition').item.json.url }}" },
			},
			{ x: 0, y: 0 },
			lookup,
		)
		const clic = triggerNode(
			{ id: 't1', kind: 'Manual' },
			{ x: 0, y: 0 },
			lookup,
		)

		const saved = toGraph([clic, source, reader], [])

		expect(saved.connectors[0].name).toBe('Condition')
		expect(saved.connectors[1].name).toBeUndefined()
		expect(saved.triggers[0].name).toBeUndefined()
	})

	it('saves a trigger under its name once a node reads it', () => {
		const clic = triggerNode(
			{ id: 't1', kind: 'Manual' },
			{ x: 0, y: 0 },
			lookup,
		)
		const reader = connectorNode(
			{
				id: 'c2',
				kind: 'http.request',
				version: 1,
				config: { url: "{{ $('Clic').item.json.url }}" },
			},
			{ x: 0, y: 0 },
			lookup,
		)

		expect(toGraph([clic, reader], []).triggers[0].name).toBe('Clic')
	})

	it('keeps the name a node was given', () => {
		const source = connectorNode(
			{ id: 'c1', kind: 'flow.if', version: 1, config: {}, name: 'Tarifs' },
			{ x: 0, y: 0 },
			lookup,
		)
		const reader = connectorNode(
			{
				id: 'c2',
				kind: 'http.request',
				version: 1,
				config: { url: "{{ $('Tarifs').item.json.url }}" },
			},
			{ x: 0, y: 0 },
			lookup,
		)

		expect(toGraph([source, reader], []).connectors[0]).toBe(
			source.data.connector,
		)
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

describe('editor nodes — the scheduler and the webhook', () => {
	const schedule: Schemas.PlacedTriggerDto = {
		id: 't1',
		kind: { Schedule: { frequency: 'Daily', time: '09:00', timezone: 'UTC' } },
	}
	const webhook: Schemas.PlacedTriggerDto = {
		id: 't2',
		kind: {
			Webhook: {
				method: 'POST',
				path: 'paiement-recu',
				credential_id: null,
				contract: [],
				on_invalid: 'Reject',
			},
		},
	}

	it('presents a scheduler with its rule as the subtitle', () => {
		const node = triggerNode(schedule, { x: 0, y: 0 }, lookup)

		expect(node.data).toMatchObject({
			kind: 'trigger',
			label: 'Planificateur',
			subtitle: 'tous les jours à 9:00',
			typeLabel: 'Planificateur',
		})
	})

	it('presents a webhook with the label and the subtitle of the prototype', () => {
		const node = triggerNode(webhook, { x: 0, y: 0 }, lookup)

		expect(node.data).toMatchObject({
			label: 'Webhook',
			subtitle: 'Déclencheur HTTP',
			typeLabel: 'Webhook',
		})
	})

	it('keeps the name and the description given to the node', () => {
		const node = withDetails(
			triggerNode(schedule, { x: 0, y: 0 }, lookup),
			{ name: 'Chaque matin', description: 'Avant l’ouverture' },
			lookup,
		)

		expect(node.data).toMatchObject({
			label: 'Chaque matin',
			subtitle: 'Avant l’ouverture',
			typeLabel: 'Planificateur',
		})
	})

	it('saves the kinds back untouched', () => {
		const nodes = toEditorNodes(
			{ triggers: [schedule, webhook], connectors: [], edges: [] },
			new Map(),
			lookup,
			new Map(),
			new Map(),
		)

		expect(toGraph(nodes, []).triggers).toEqual([schedule, webhook])
	})
})
