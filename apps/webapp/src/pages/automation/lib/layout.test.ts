import { describe, expect, it } from 'vitest'
import type { Graph, PlacedConnector } from '#/hooks/use-automation'
import { layoutPositions } from '#/pages/automation/lib/layout'

function placed(id: string): PlacedConnector {
	return { id, kind: 'http.request', version: 1, config: {} }
}

describe('layoutPositions — column by depth', () => {
	it('places an isolated connector at column 0', () => {
		const graph: Graph = { connectors: [placed('a')], edges: [] }
		expect(layoutPositions(graph).get('a')).toEqual({ x: 0, y: 0 })
	})

	it('advances one column per edge along a linear chain', () => {
		const graph: Graph = {
			connectors: [placed('a'), placed('b'), placed('c')],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c' },
			],
		}

		const positions = layoutPositions(graph)

		expect(positions.get('a')?.x).toBe(0)
		expect(positions.get('b')?.x).toBeGreaterThan(positions.get('a')?.x ?? 0)
		expect(positions.get('c')?.x).toBeGreaterThan(positions.get('b')?.x ?? 0)
	})

	it('a diamond takes the longest path into the merge point', () => {
		const graph: Graph = {
			connectors: [placed('a'), placed('b'), placed('c'), placed('d')],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
				{ from: 'b', to: 'd' },
				{ from: 'c', to: 'd' },
			],
		}

		const positions = layoutPositions(graph)

		expect(positions.get('b')?.x).toBe(positions.get('c')?.x)
		expect(positions.get('d')?.x).toBeGreaterThan(positions.get('b')?.x ?? 0)
	})

	it('branches sharing a column get distinct rows, never overlapping', () => {
		const graph: Graph = {
			connectors: [placed('a'), placed('b'), placed('c')],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
			],
		}

		const positions = layoutPositions(graph)

		expect(positions.get('b')?.x).toBe(positions.get('c')?.x)
		expect(positions.get('b')?.y).not.toBe(positions.get('c')?.y)
	})

	it('terminates and still positions every connector when the graph has a cycle', () => {
		const graph: Graph = {
			connectors: [placed('a'), placed('b')],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'a' },
			],
		}

		const positions = layoutPositions(graph)

		expect(positions.size).toBe(2)
		expect(positions.get('a')).toBeDefined()
		expect(positions.get('b')).toBeDefined()
	})
})
