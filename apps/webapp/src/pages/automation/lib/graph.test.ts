import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorsReferencing,
	copyLabel,
	findFreePosition,
	isBranchDeclared,
	neighboursOf,
	nextConnectorId,
	nextTriggerId,
	placementAfter,
	readLayout,
	reconnectionsAfterDelete,
	removeConnector,
	removeTrigger,
	rootConnectorIds,
	snapToGrid,
	upstreamConnectorIds,
	upstreamOf,
} from '#/pages/automation/lib/graph'

function connector(
	id: string,
	config: Record<string, unknown> = {},
): Schemas.PlacedConnectorDto {
	return { id, kind: 'http.request', version: 1, config }
}

function graphOf(
	connectors: Schemas.PlacedConnectorDto[],
	edges: Schemas.EdgeDto[] = [],
): Schemas.GraphDto {
	return { connectors, edges, triggers: [] }
}

function triggeredGraphOf(
	connectors: Schemas.PlacedConnectorDto[],
	edges: Schemas.EdgeDto[] = [],
): Schemas.GraphDto {
	return {
		connectors,
		edges: [{ from: 't1', to: connectors[0].id, branch: null }, ...edges],
		triggers: [{ id: 't1', kind: 'Manual' }],
	}
}

describe('readLayout', () => {
	it('keeps the entries that are points', () => {
		const layout = readLayout({ c1: { x: 10.5, y: -20 } })

		expect(layout.get('c1')).toEqual({ x: 10.5, y: -20 })
	})

	it('is empty when there is no layout at all', () => {
		expect(readLayout(null).size).toBe(0)
		expect(readLayout(undefined).size).toBe(0)
	})

	it('drops an entry that is not a point instead of failing', () => {
		const layout = readLayout({
			c1: { x: 1, y: 2 },
			c2: 'not a point',
			c3: { x: 3 },
			c4: { x: 'left', y: 2 },
		})

		expect([...layout.keys()]).toEqual(['c1'])
	})
})

describe('nextConnectorId', () => {
	it('starts at c1 on an empty graph', () => {
		expect(nextConnectorId(graphOf([]))).toBe('c1')
	})

	it('takes the next number after the highest one in use', () => {
		expect(nextConnectorId(graphOf([connector('c1'), connector('c2')]))).toBe(
			'c3',
		)
	})

	it('never reuses a freed id while a higher one survives', () => {
		expect(nextConnectorId(graphOf([connector('c1'), connector('c5')]))).toBe(
			'c6',
		)
	})

	it('ignores ids that carry no number', () => {
		expect(
			nextConnectorId(graphOf([connector('start'), connector('c2')])),
		).toBe('c3')
	})
})

function graphWithTriggers(
	triggers: Schemas.PlacedTriggerDto[],
): Schemas.GraphDto {
	return { connectors: [], edges: [], triggers }
}

describe('nextTriggerId', () => {
	it('starts at t1 on an empty graph', () => {
		expect(nextTriggerId(graphWithTriggers([]))).toBe('t1')
	})

	it('takes the next number after the highest one in use', () => {
		expect(
			nextTriggerId(
				graphWithTriggers([
					{ id: 't1', kind: 'Manual' },
					{ id: 't2', kind: 'Manual' },
				]),
			),
		).toBe('t3')
	})

	it('never reuses a freed id while a higher one survives', () => {
		expect(
			nextTriggerId(
				graphWithTriggers([
					{ id: 't1', kind: 'Manual' },
					{ id: 't5', kind: 'Manual' },
				]),
			),
		).toBe('t6')
	})

	it('ignores ids that carry no number', () => {
		expect(
			nextTriggerId(
				graphWithTriggers([
					{ id: 'start', kind: 'Manual' },
					{ id: 't2', kind: 'Manual' },
				]),
			),
		).toBe('t3')
	})
})

describe('rootConnectorIds', () => {
	it('still finds the root a trigger points at', () => {
		const graph = triggeredGraphOf(
			[connector('c1'), connector('c2')],
			[{ from: 'c1', to: 'c2', branch: null }],
		)

		expect(rootConnectorIds(graph)).toEqual(['c1'])
	})

	it('names every connector no edge points at', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3')],
			[{ from: 'c1', to: 'c2' }],
		)

		expect(rootConnectorIds(graph)).toEqual(['c1', 'c3'])
	})

	it('counts a connector dropped without any edge as a root', () => {
		expect(rootConnectorIds(graphOf([connector('c1')]))).toEqual(['c1'])
	})
})

describe('removeConnector', () => {
	it('removes the connector and every edge touching it', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3')],
			[
				{ from: 'c1', to: 'c2' },
				{ from: 'c2', to: 'c3' },
				{ from: 'c1', to: 'c3' },
			],
		)

		const next = removeConnector(graph, 'c2')

		expect(next.connectors.map((c) => c.id)).toEqual(['c1', 'c3'])
		expect(next.edges).toEqual([{ from: 'c1', to: 'c3' }])
	})

	it('leaves the neighbours unwired rather than stitching them together', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3')],
			[
				{ from: 'c1', to: 'c2' },
				{ from: 'c2', to: 'c3' },
			],
		)

		expect(removeConnector(graph, 'c2').edges).toEqual([])
	})
})

describe('removeTrigger', () => {
	it('removes the trigger and every edge leaving it', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [
				{ from: 't1', to: 'c1', branch: null },
				{ from: 't2', to: 'c2', branch: null },
			],
			triggers: [
				{ id: 't1', kind: 'Manual' },
				{ id: 't2', kind: { Events: ['quote.accepted'] } },
			],
		}

		const next = removeTrigger(graph, 't1')

		expect(next.triggers).toEqual([
			{ id: 't2', kind: { Events: ['quote.accepted'] } },
		])
		expect(next.edges).toEqual([{ from: 't2', to: 'c2', branch: null }])
	})

	it('leaves the other trigger, its edges, and every connector untouched', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1'), connector('c2')],
			edges: [
				{ from: 't1', to: 'c1', branch: null },
				{ from: 't2', to: 'c2', branch: null },
				{ from: 'c1', to: 'c2', branch: null },
			],
			triggers: [
				{ id: 't1', kind: 'Manual' },
				{ id: 't2', kind: 'Manual' },
			],
		}

		const next = removeTrigger(graph, 't1')

		expect(next.connectors).toEqual(graph.connectors)
		expect(next.edges).toEqual([
			{ from: 't2', to: 'c2', branch: null },
			{ from: 'c1', to: 'c2', branch: null },
		])
	})
})

describe('connectorsReferencing', () => {
	it('names the connectors whose config reads the given one', () => {
		const graph = graphOf([
			connector('c1'),
			connector('c2', { url: '{{ connectors.c1.output.href }}' }),
			connector('c3', { body: { nested: '{{ connectors.c1.output.id }}' } }),
			connector('c4', { url: '{{ connectors.c2.output.href }}' }),
		])

		expect(connectorsReferencing(graph, 'c1')).toEqual(['c2', 'c3'])
	})

	it('does not match a connector whose id merely starts the same', () => {
		const graph = graphOf([
			connector('c1'),
			connector('c2', { url: '{{ connectors.c10.output.href }}' }),
		])

		expect(connectorsReferencing(graph, 'c1')).toEqual([])
	})

	it('is empty when nothing references it', () => {
		expect(connectorsReferencing(graphOf([connector('c1')]), 'c1')).toEqual([])
	})
})

describe('upstreamConnectorIds', () => {
	it('never names a trigger among the upstream connectors', () => {
		const graph = triggeredGraphOf(
			[connector('c1'), connector('c2')],
			[{ from: 'c1', to: 'c2', branch: null }],
		)

		expect(upstreamConnectorIds(graph, 'c1')).toEqual([])
		expect(upstreamConnectorIds(graph, 'c2')).toEqual(['c1'])
	})

	it('names every ancestor, closest first, of a deep chain', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3')],
			[
				{ from: 'c1', to: 'c2' },
				{ from: 'c2', to: 'c3' },
			],
		)

		expect(upstreamConnectorIds(graph, 'c3')).toEqual(['c2', 'c1'])
	})

	it('excludes a sibling reachable only through a different branch', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3')],
			[
				{ from: 'c1', to: 'c2', branch: 'Then' },
				{ from: 'c1', to: 'c3', branch: 'Else' },
			],
		)

		expect(upstreamConnectorIds(graph, 'c2')).toEqual(['c1'])
		expect(upstreamConnectorIds(graph, 'c3')).toEqual(['c1'])
	})

	it('is empty for a root connector', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2')],
			[{ from: 'c1', to: 'c2' }],
		)

		expect(upstreamConnectorIds(graph, 'c1')).toEqual([])
	})

	it('visits a diamond merge exactly once', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2'), connector('c3'), connector('c4')],
			[
				{ from: 'c1', to: 'c2' },
				{ from: 'c1', to: 'c3' },
				{ from: 'c2', to: 'c4' },
				{ from: 'c3', to: 'c4' },
			],
		)

		expect(upstreamConnectorIds(graph, 'c4').sort()).toEqual(['c1', 'c2', 'c3'])
	})

	it('does not loop forever on a cycle', () => {
		const graph = graphOf(
			[connector('c1'), connector('c2')],
			[
				{ from: 'c1', to: 'c2' },
				{ from: 'c2', to: 'c1' },
			],
		)

		expect(upstreamConnectorIds(graph, 'c2').sort()).toEqual(['c1', 'c2'])
	})
})

describe('snapToGrid', () => {
	it('rounds a position onto the 20px grid', () => {
		expect(snapToGrid({ x: 29, y: 31 })).toEqual({ x: 20, y: 40 })
		expect(snapToGrid({ x: -9, y: 0 })).toEqual({ x: -0, y: 0 })
	})
})

describe('copyLabel', () => {
	it('marks a copy', () => {
		expect(copyLabel('Créer un projet')).toBe('Créer un projet (copie)')
	})
})

describe('findFreePosition', () => {
	it('slides down until the slot is free', () => {
		const taken = [{ position: { x: 0, y: 0 } }, { position: { x: 0, y: 140 } }]
		expect(findFreePosition({ x: 0, y: 0 }, taken)).toEqual({ x: 0, y: 280 })
	})

	it('keeps a free origin', () => {
		expect(
			findFreePosition({ x: 400, y: 0 }, [{ position: { x: 0, y: 0 } }]),
		).toEqual({
			x: 400,
			y: 0,
		})
	})
})

describe('placementAfter', () => {
	it('puts a single output one column to the right', () => {
		expect(placementAfter({ x: 100, y: 40 }, 0, 1)).toEqual({ x: 320, y: 40 })
	})

	it('spreads the branches around the source', () => {
		expect(placementAfter({ x: 0, y: 0 }, 0, 2).y).toBe(-70)
		expect(placementAfter({ x: 0, y: 0 }, 1, 2).y).toBe(70)
	})
})

describe('reconnectionsAfterDelete', () => {
	it('stitches the neighbours of a deleted middle node, keeping the branch', () => {
		expect(
			reconnectionsAfterDelete(
				['b'],
				[
					{ source: 'a', target: 'b', sourceHandle: 'Then' },
					{ source: 'b', target: 'c' },
				],
				[],
			),
		).toEqual([{ source: 'a', sourceHandle: 'Then', target: 'c' }])
	})

	it('does not stitch a node with several inputs', () => {
		expect(
			reconnectionsAfterDelete(
				['b'],
				[
					{ source: 'a', target: 'b' },
					{ source: 'x', target: 'b' },
					{ source: 'b', target: 'c' },
				],
				[],
			),
		).toEqual([])
	})

	it('does not duplicate an edge that already exists', () => {
		expect(
			reconnectionsAfterDelete(
				['b'],
				[
					{ source: 'a', target: 'b' },
					{ source: 'b', target: 'c' },
				],
				[{ source: 'a', target: 'c' }],
			),
		).toEqual([])
	})
})

describe('isBranchDeclared', () => {
	const descriptors = new Map<string, Schemas.ConnectorDescriptorResponse>([
		[
			'flow.if',
			{
				kind: 'flow.if',
				label: 'Si',
				family: 'flow',
				version: 1,
				auth: 'None',
				fields: [],
				branches: ['Then', 'Else'],
				output_example: null,
			},
		],
		[
			'http.request',
			{
				kind: 'http.request',
				label: 'HTTP',
				family: 'http',
				version: 1,
				auth: 'None',
				fields: [],
				branches: [],
				output_example: null,
			},
		],
	])
	const graph: Schemas.GraphDto = {
		triggers: [{ id: 't1', kind: 'Manual' }],
		connectors: [
			{ id: 'c1', kind: 'flow.if', version: 1, config: {} },
			{ id: 'c2', kind: 'http.request', version: 1, config: {} },
		],
		edges: [],
	}

	it('accepts a plain edge out of a trigger and an unbranched connector', () => {
		expect(
			isBranchDeclared({ source: 't1', target: 'c1' }, graph, descriptors),
		).toBe(true)
		expect(
			isBranchDeclared({ source: 'c2', target: 'c1' }, graph, descriptors),
		).toBe(true)
	})

	it('requires a declared branch on a branched connector', () => {
		expect(
			isBranchDeclared(
				{ source: 'c1', target: 'c2', sourceHandle: 'Then' },
				graph,
				descriptors,
			),
		).toBe(true)
		expect(
			isBranchDeclared(
				{ source: 'c1', target: 'c2', sourceHandle: 'Each' },
				graph,
				descriptors,
			),
		).toBe(false)
		expect(
			isBranchDeclared({ source: 'c1', target: 'c2' }, graph, descriptors),
		).toBe(false)
	})

	it('refuses a branch out of a trigger or an unbranched connector', () => {
		expect(
			isBranchDeclared(
				{ source: 't1', target: 'c1', sourceHandle: 'Then' },
				graph,
				descriptors,
			),
		).toBe(false)
		expect(
			isBranchDeclared(
				{ source: 'c2', target: 'c1', sourceHandle: 'Then' },
				graph,
				descriptors,
			),
		).toBe(false)
	})
})

describe('upstreamOf', () => {
	it('walks back through the edges, nearest first', () => {
		expect(
			upstreamOf('c3', [
				{ source: 't1', target: 'c1' },
				{ source: 'c1', target: 'c2', sourceHandle: 'Then' },
				{ source: 'c2', target: 'c3' },
			]),
		).toEqual([
			{ id: 'c2', distance: 1, handle: null },
			{ id: 'c1', distance: 2, handle: 'Then' },
			{ id: 't1', distance: 3, handle: null },
		])
	})

	it('survives a cycle and never lists the node itself', () => {
		expect(
			upstreamOf('a', [
				{ source: 'a', target: 'b' },
				{ source: 'b', target: 'a' },
			]).map((link) => link.id),
		).toEqual(['b'])
	})
})

describe('neighboursOf', () => {
	it('splits the previous and next nodes, keeping the branch', () => {
		expect(
			neighboursOf('b', [
				{ source: 'a', target: 'b' },
				{ source: 'b', target: 'c', sourceHandle: 'Else' },
			]),
		).toEqual({
			previous: [{ id: 'a', handle: null }],
			next: [{ id: 'c', handle: 'Else' }],
		})
	})
})
