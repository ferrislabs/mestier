import { describe, expect, it } from 'vitest'
import { surveyNeighboursOf } from '#/pages/survey/lib/survey-neighbours'
import type { EditorGraph } from '#/pages/survey/types'

const GRAPH: EditorGraph = {
	start: 'q1',
	nodes: [
		{
			type: 'QUESTION',
			id: 'q1',
			kind: 'SINGLE_CHOICE',
			label: 'Voiture ?',
			required: true,
			options: ['Oui', 'Non'],
		},
		{
			type: 'CONDITION',
			id: 'c1',
			question: 'q1',
			option: 'Oui',
			checked: true,
		},
		{
			type: 'QUESTION',
			id: 'q2',
			kind: 'TEXT',
			label: 'Plaque',
			required: false,
		},
		{ type: 'END', id: 'e1', message: null },
	],
	edges: [
		{ from: 'q1', to: 'c1', branch: 'NEXT' },
		{ from: 'c1', to: 'e1', branch: 'ELSE' },
		{ from: 'c1', to: 'q2', branch: 'THEN' },
		{ from: 'q2', to: 'e1', branch: 'NEXT' },
	],
}

describe('surveyNeighboursOf', () => {
	it('gives the first node no previous neighbour and its successor as next', () => {
		expect(surveyNeighboursOf(GRAPH, 'q1')).toEqual({
			previous: [],
			next: [{ id: 'c1', branch: 'NEXT' }],
		})
	})

	it('lists the two branches of a condition, Oui first', () => {
		expect(surveyNeighboursOf(GRAPH, 'c1').next).toEqual([
			{ id: 'q2', branch: 'THEN' },
			{ id: 'e1', branch: 'ELSE' },
		])
	})

	it('gives an end every node pointing to it as previous and no next', () => {
		expect(surveyNeighboursOf(GRAPH, 'e1')).toEqual({
			previous: [
				{ id: 'c1', branch: 'ELSE' },
				{ id: 'q2', branch: 'NEXT' },
			],
			next: [],
		})
	})

	it('ignores edges towards a node that no longer exists', () => {
		const graph = {
			...GRAPH,
			edges: [
				...GRAPH.edges,
				{ from: 'e1', to: 'ghost', branch: 'NEXT' as const },
			],
		}
		expect(surveyNeighboursOf(graph, 'e1').next).toEqual([])
	})
})
