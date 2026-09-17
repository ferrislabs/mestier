import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	computeFallbackLayout,
	mergedLayout,
} from '#/pages/automation/lib/layout'

function connector(id: string): Schemas.PlacedConnectorDto {
	return { id, kind: 'http.request', version: 1, config: {} }
}

function graphOf(
	connectors: Schemas.PlacedConnectorDto[],
	edges: Schemas.EdgeDto[] = [],
): Schemas.GraphDto {
	return { connectors, edges, triggers: [] }
}

describe('computeFallbackLayout', () => {
	it('places a linear chain left to right, one column per hop', () => {
		const graph = graphOf(
			[connector('a'), connector('b'), connector('c')],
			[
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c' },
			],
		)

		const layout = computeFallbackLayout(graph, new Set())

		expect(layout.get('a')).toEqual({ x: 0, y: 0 })
		expect(layout.get('b')).toEqual({ x: 280, y: 0 })
		expect(layout.get('c')).toEqual({ x: 560, y: 0 })
	})

	it('stacks siblings in the same column at increasing rows, in edge order', () => {
		const graph = graphOf(
			[connector('a'), connector('b'), connector('c')],
			[
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
			],
		)

		const layout = computeFallbackLayout(graph, new Set())

		expect(layout.get('a')).toEqual({ x: 0, y: 0 })
		expect(layout.get('b')).toEqual({ x: 280, y: 0 })
		expect(layout.get('c')).toEqual({ x: 280, y: 140 })
	})

	it('places a diamond merge at the longest path from the root', () => {
		const graph = graphOf(
			[connector('a'), connector('b'), connector('c'), connector('d')],
			[
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
				{ from: 'b', to: 'd' },
				{ from: 'c', to: 'd' },
			],
		)

		const layout = computeFallbackLayout(graph, new Set())

		expect(layout.get('a')).toEqual({ x: 0, y: 0 })
		expect(layout.get('b')).toEqual({ x: 280, y: 0 })
		expect(layout.get('c')).toEqual({ x: 280, y: 140 })
		expect(layout.get('d')).toEqual({ x: 560, y: 0 })
	})

	it('gives every independent root its own row in the first column', () => {
		const graph = graphOf([connector('a'), connector('z')])

		const layout = computeFallbackLayout(graph, new Set())

		expect(layout.get('a')).toEqual({ x: 0, y: 0 })
		expect(layout.get('z')).toEqual({ x: 0, y: 140 })
	})

	it('terminates and places every connector on a graph with a cycle', () => {
		const graph = graphOf(
			[connector('a'), connector('b'), connector('c')],
			[
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c' },
				{ from: 'c', to: 'a' },
			],
		)

		const layout = computeFallbackLayout(graph, new Set())

		expect(layout.size).toBe(3)
		for (const id of ['a', 'b', 'c']) {
			const position = layout.get(id)
			expect(position).toBeDefined()
			expect(Number.isFinite(position?.x)).toBe(true)
			expect(Number.isFinite(position?.y)).toBe(true)
		}
	})

	it('never places a connector whose id is already covered by the stored layout', () => {
		const graph = graphOf(
			[connector('a'), connector('b'), connector('c')],
			[
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c' },
			],
		)

		const layout = computeFallbackLayout(graph, new Set(['b']))

		expect([...layout.keys()]).toEqual(['a', 'c'])
	})

	it('is empty when every connector already has a stored position', () => {
		const graph = graphOf([connector('a')])

		const layout = computeFallbackLayout(graph, new Set(['a']))

		expect(layout.size).toBe(0)
	})
})

describe('mergedLayout', () => {
	it('keeps the stored position for a connector that has one', () => {
		const graph = graphOf([connector('a')])

		const layout = mergedLayout(graph, new Map([['a', { x: 40, y: 20 }]]))

		expect(layout.get('a')).toEqual({ x: 40, y: 20 })
	})

	it('falls back to a computed position for a connector with none stored', () => {
		const graph = graphOf(
			[connector('a'), connector('b')],
			[{ from: 'a', to: 'b' }],
		)

		const layout = mergedLayout(graph, new Map([['a', { x: 40, y: 20 }]]))

		expect(layout.get('a')).toEqual({ x: 40, y: 20 })
		expect(layout.get('b')).toEqual(
			computeFallbackLayout(graph, new Set(['a'])).get('b'),
		)
	})
})
