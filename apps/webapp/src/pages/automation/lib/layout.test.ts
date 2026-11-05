import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	computeFallbackLayout,
	mergedLayout,
	tidyLayout,
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

describe('tidyLayout', () => {
	const node = (
		id: string,
		x: number,
		y: number,
		outputs: string[] = [''],
	) => ({
		id,
		position: { x, y },
		trigger: id.startsWith('t'),
		outputs,
	})

	it('lines a chain up in columns on the grid', () => {
		const result = tidyLayout(
			[node('t1', 3, 7), node('c1', 400, 90), node('c2', 50, 500)],
			[
				{ source: 't1', target: 'c1' },
				{ source: 'c1', target: 'c2' },
			],
		)
		expect(result.get('t1')).toEqual({ x: 0, y: 0 })
		expect(result.get('c1')).toEqual({ x: 220, y: 0 })
		expect(result.get('c2')).toEqual({ x: 440, y: 0 })
	})

	it('spreads the two branches of a node on both sides of its row', () => {
		const result = tidyLayout(
			[node('c1', 0, 0, ['Then', 'Else']), node('c2', 0, 0), node('c3', 0, 0)],
			[
				{ source: 'c1', target: 'c2', sourceHandle: 'Then' },
				{ source: 'c1', target: 'c3', sourceHandle: 'Else' },
			],
		)
		const then = result.get('c2')
		const otherwise = result.get('c3')
		expect(then && otherwise && then.y < otherwise.y).toBe(true)
	})

	it('stacks unrelated components under each other', () => {
		const result = tidyLayout([node('t1', 0, 0), node('c1', 0, 0)], [])
		expect(result.get('t1')?.y).toBe(0)
		expect(result.get('c1')?.y).toBe(200)
	})

	it('is empty for no nodes', () => {
		expect(tidyLayout([], []).size).toBe(0)
	})
})
