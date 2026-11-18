import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { validateGraph } from '#/pages/survey/lib/graph-validation'
import {
	addNode,
	answeredPath,
	connect,
	createNode,
	disconnect,
	emptyGraph,
	fromSchema,
	openExit,
	precedingChoiceQuestions,
	removeNodes,
	START_HANDLE,
	START_ID,
	toSchema,
} from '#/pages/survey/lib/survey-graph'

interface PathCase {
	name: string
	graph: Schemas.SurveyGraphSchema
	answers: Record<string, unknown>
	expected_questions: string[]
}

const CASES: PathCase[] = JSON.parse(
	readFileSync(
		resolve(
			import.meta.dirname,
			'../../../../../../libs/core/src/domain/survey/fixtures/paths.json',
		),
		'utf8',
	),
)

describe('shared path cases', () => {
	it.each(
		CASES.map((item) => [item.name, item] as const),
	)('%s follows the same path as the backend and passes the validator', (_, item) => {
		const graph = fromSchema(item.graph)
		expect(validateGraph(graph)).toEqual([])
		expect(answeredPath(graph, item.answers).map((node) => node.id)).toEqual(
			item.expected_questions,
		)
	})
})

describe('graph edition', () => {
	it('makes the first question the start and links a node after a free exit', () => {
		const first = addNode(emptyGraph(), createNode('TEXT', 'q1'), null)
		expect(first.start).toBe('q1')
		const second = addNode(first, createNode('END', 'e1'), {
			from: 'q1',
			handle: 'NEXT',
		})
		expect(second.edges).toEqual([{ from: 'q1', to: 'e1', branch: 'NEXT' }])
		expect(openExit(second, 'q1')).toBeNull()
		expect(openExit(second, START_ID)).toBeNull()
	})

	it('replaces the edge of a branch instead of adding a second one', () => {
		let graph = addNode(emptyGraph(), createNode('TEXT', 'q1'), null)
		graph = addNode(graph, createNode('END', 'e1'), null)
		graph = addNode(graph, createNode('END', 'e2'), null)
		graph = connect(graph, 'q1', 'NEXT', 'e1')
		graph = connect(graph, 'q1', 'NEXT', 'e2')
		expect(graph.edges).toEqual([{ from: 'q1', to: 'e2', branch: 'NEXT' }])
	})

	it('refuses a branch the node does not have and a link to itself', () => {
		const graph = addNode(emptyGraph(), createNode('TEXT', 'q1'), null)
		expect(connect(graph, 'q1', 'THEN', 'q1')).toBe(graph)
		expect(connect(graph, 'q1', 'THEN', 'q2')).toBe(graph)
	})

	it('moves and clears the start through the start node', () => {
		let graph = addNode(emptyGraph(), createNode('TEXT', 'q1'), null)
		graph = addNode(graph, createNode('TEXT', 'q2'), null)
		graph = connect(graph, START_ID, START_HANDLE, 'q2')
		expect(graph.start).toBe('q2')
		expect(disconnect(graph, START_ID, START_HANDLE).start).toBeNull()
	})

	it('drops the edges and the start of a removed node', () => {
		let graph = addNode(emptyGraph(), createNode('TEXT', 'q1'), null)
		graph = addNode(graph, createNode('END', 'e1'), {
			from: 'q1',
			handle: 'NEXT',
		})
		const removed = removeNodes(graph, new Set(['q1']))
		expect(removed).toEqual({
			start: null,
			nodes: [{ type: 'END', id: 'e1', message: null }],
			edges: [],
		})
	})

	it('serialises a clean graph and nothing for an empty one', () => {
		let graph = addNode(
			emptyGraph(),
			{ ...createNode('TEXT', 'q1'), label: ' Nom ' } as never,
			null,
		)
		graph = addNode(graph, createNode('END', 'e1'), {
			from: 'q1',
			handle: 'NEXT',
		})
		expect(toSchema(emptyGraph())).toBeNull()
		expect(toSchema(graph)).toEqual({
			start: 'q1',
			nodes: [
				{
					type: 'QUESTION',
					id: 'q1',
					kind: 'TEXT',
					label: 'Nom',
					required: false,
					options: [],
					max_photos: null,
					identifier: false,
				},
				{ type: 'END', id: 'e1', message: null },
			],
			edges: [{ from: 'q1', to: 'e1', branch: 'NEXT' }],
		})
	})

	it('offers the choice questions that come before a condition', () => {
		let graph = addNode(emptyGraph(), createNode('SINGLE_CHOICE', 'q1'), null)
		graph = addNode(graph, createNode('CONDITION', 'c1'), {
			from: 'q1',
			handle: 'NEXT',
		})
		graph = addNode(graph, createNode('MULTIPLE_CHOICE', 'q2'), {
			from: 'c1',
			handle: 'THEN',
		})
		expect(
			precedingChoiceQuestions(graph, 'c1').map((node) => node.id),
		).toEqual(['q1'])
	})
})
