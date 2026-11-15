import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { validateGraph } from '#/pages/survey/lib/graph-validation'
import {
	answeredPath,
	emptyGraph,
	fromSchema,
	toSchema,
} from '#/pages/survey/lib/survey-graph'
import type { EditorGraph } from '#/pages/survey/types'

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

describe('graph serialisation', () => {
	it('serialises a clean graph and nothing for an empty one', () => {
		const graph: EditorGraph = {
			start: 'q1',
			nodes: [
				{
					type: 'QUESTION',
					id: 'q1',
					kind: 'TEXT',
					label: ' Nom ',
					required: false,
					options: ['ignorée'],
				},
				{ type: 'END', id: 'e1', message: '  ' },
			],
			edges: [{ from: 'q1', to: 'e1', branch: 'NEXT' }],
		}
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

	it('trims a description and drops a blank or null one', () => {
		const graph = (description: string | null): EditorGraph => ({
			start: 'q1',
			nodes: [
				{
					type: 'QUESTION',
					id: 'q1',
					kind: 'TEXT',
					label: 'Nom',
					required: false,
					description,
				},
				{ type: 'END', id: 'e1', message: null },
			],
			edges: [{ from: 'q1', to: 'e1', branch: 'NEXT' }],
		})
		const described = (description: string | null) =>
			toSchema(graph(description))?.nodes[0]
		expect(described('  Une ligne\nPuis une autre  ')).toMatchObject({
			description: 'Une ligne\nPuis une autre',
		})
		expect(described('   ')).not.toHaveProperty('description')
		expect(described(null)).not.toHaveProperty('description')
	})
})
