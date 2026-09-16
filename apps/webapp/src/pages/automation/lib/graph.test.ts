import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorsReferencing,
	nextConnectorId,
	nextNodePosition,
	readLayout,
	removeConnector,
	rootConnectorIds,
	upstreamConnectorIds,
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
	return { connectors, edges }
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

describe('rootConnectorIds', () => {
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

describe('nextNodePosition', () => {
	it('places the new node one column to the right of its source', () => {
		expect(nextNodePosition({ x: 100, y: 40 }, 0)).toEqual({ x: 380, y: 40 })
	})

	it('offsets further down the same column for each existing sibling', () => {
		expect(nextNodePosition({ x: 0, y: 0 }, 2)).toEqual({ x: 280, y: 280 })
	})
})
