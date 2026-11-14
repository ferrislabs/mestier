import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { fromSchema } from '#/pages/survey/lib/survey-graph'
import {
	knownLength,
	respondPath,
} from '#/pages/survey-respond/lib/respond-path'
import { branchingGraph } from '#/pages/survey-respond/test/respond-fixtures'

interface PathCase {
	name: string
	graph: Schemas.SurveyGraphSchema
	answers: Record<string, unknown>
	expected_questions: string[]
	expected_end: string | null
}

const CASES: PathCase[] = JSON.parse(
	readFileSync(
		resolve(
			process.cwd(),
			'../../libs/core/src/domain/survey/fixtures/paths.json',
		),
		'utf8',
	),
)

describe('respondPath', () => {
	it.each(
		CASES.map((item) => [item.name, item] as const),
	)('%s reaches the same questions and end as the backend', (_, item) => {
		const path = respondPath(fromSchema(item.graph), item.answers)
		expect(path.questions.map((node) => node.id)).toEqual(
			item.expected_questions,
		)
		expect(path.end?.id ?? null).toEqual(item.expected_end)
	})
})

describe('knownLength', () => {
	it('is unknown while a condition lies ahead and known after it', () => {
		const graph = fromSchema(branchingGraph())
		expect(knownLength(graph, 'q-role', 3)).toBeNull()
		expect(knownLength(graph, 'q-email', 3)).toBe(3)
	})
})
