import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildGraph,
	conditionNodeId,
	conditionOwners,
	parseGraph,
} from '#/pages/survey/lib/form-graph'
import { validateGraph } from '#/pages/survey/lib/graph-validation'
import { QUESTION_KINDS } from '#/pages/survey/lib/question-kinds'
import {
	type DisplayRule,
	type FormQuestion,
	newQuestion,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import {
	answeredPath,
	fromSchema,
	normalizeGraph,
} from '#/pages/survey/lib/survey-graph'
import type { EditorGraph, QuestionKind } from '#/pages/survey/types'

const END = '00000000-0000-4000-8000-0000000000ee'

function question(
	id: string,
	kind: QuestionKind,
	patch: Partial<FormQuestion> = {},
): FormQuestion {
	return { ...newQuestion(id, kind), label: `Question ${id}`, ...patch }
}

function form(questions: FormQuestion[], endMessage = ''): SurveyForm {
	return { questions, endId: END, endMessage }
}

function normalized(value: SurveyForm): SurveyForm {
	return {
		...value,
		questions: value.questions.map((item) => ({
			...item,
			rule:
				item.rule && item.rule.conditions.length === 1
					? { ...item.rule, join: 'ALL' }
					: item.rule,
		})),
	}
}

function roundTrip(value: SurveyForm) {
	return parseGraph(buildGraph(value))
}

const ALL_TWO: DisplayRule = {
	join: 'ALL',
	conditions: [
		{ question: 'a', option: 'Oui', checked: true },
		{ question: 'b', option: 'Rouge', checked: false },
	],
}

const CHOICES = {
	a: question('a', 'SINGLE_CHOICE', { options: ['Oui', 'Non'] }),
	b: question('b', 'MULTIPLE_CHOICE', { options: ['Rouge', 'Vert'] }),
}

const TABLE: [string, SurveyForm][] = [
	['an empty form', form([])],
	['an empty form with an end message', form([], 'Merci')],
	[
		'questions without rules',
		form([question('a', 'TEXT'), question('b', 'NUMBER')], 'Merci'),
	],
	[
		'one rule',
		form([
			CHOICES.a,
			question('c', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		]),
	],
	[
		'all of two conditions',
		form([CHOICES.a, CHOICES.b, question('c', 'TEXT', { rule: ALL_TWO })]),
	],
	[
		'any of two conditions',
		form([
			CHOICES.a,
			CHOICES.b,
			question('c', 'TEXT', { rule: { ...ALL_TWO, join: 'ANY' } }),
			question('d', 'DATE'),
		]),
	],
	[
		'mixed kinds',
		form(
			[
				question('a', 'TEXT', { identifier: true, required: true }),
				question('b', 'LONG_TEXT'),
				question('c', 'SINGLE_CHOICE', { options: ['Oui', 'Non'] }),
				question('d', 'PHOTO', { maxPhotos: 3 }),
				question('e', 'EMAIL', {
					rule: {
						join: 'ALL',
						conditions: [{ question: 'c', option: 'Non', checked: true }],
					},
				}),
				question('f', 'DATETIME'),
				question('g', 'PHONE'),
			],
			'Au revoir',
		),
	],
	[
		'two consecutive questions with rules',
		form([
			CHOICES.a,
			CHOICES.b,
			question('c', 'TEXT', { rule: ALL_TWO }),
			question('d', 'TEXT', { rule: { ...ALL_TWO, join: 'ANY' } }),
		]),
	],
]

describe('form round trip', () => {
	it.each(TABLE)('keeps %s through the graph', (_, value) => {
		expect(roundTrip(value)).toEqual({ kind: 'form', form: value })
	})

	it('builds an identical graph for an unchanged form', () => {
		const value = TABLE[4][1]
		expect(buildGraph(value)).toEqual(buildGraph(structuredClone(value)))
	})

	it('reads a single condition as an ALL rule', () => {
		const value = form([
			CHOICES.a,
			question('c', 'TEXT', {
				rule: {
					join: 'ANY',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		])
		expect(roundTrip(value)).toEqual({ kind: 'form', form: normalized(value) })
	})
})

describe('graph shapes', () => {
	it('chains conditions of an ALL rule through Then and skips on Else', () => {
		const graph = buildGraph(
			form([CHOICES.a, CHOICES.b, question('c', 'TEXT', { rule: ALL_TWO })]),
		)
		const first = conditionNodeId('c', 0)
		const second = conditionNodeId('c', 1)
		expect(graph.start).toBe('a')
		expect(graph.edges).toEqual(
			expect.arrayContaining([
				{ from: 'b', to: first, branch: 'NEXT' },
				{ from: first, to: second, branch: 'THEN' },
				{ from: first, to: END, branch: 'ELSE' },
				{ from: second, to: 'c', branch: 'THEN' },
				{ from: second, to: END, branch: 'ELSE' },
				{ from: 'c', to: END, branch: 'NEXT' },
			]),
		)
		expect(graph.edges).toHaveLength(7)
	})

	it('chains conditions of an ANY rule through Else with every Then to the question', () => {
		const graph = buildGraph(
			form([
				CHOICES.a,
				CHOICES.b,
				question('c', 'TEXT', { rule: { ...ALL_TWO, join: 'ANY' } }),
				question('d', 'TEXT'),
			]),
		)
		const first = conditionNodeId('c', 0)
		const second = conditionNodeId('c', 1)
		expect(graph.edges).toEqual(
			expect.arrayContaining([
				{ from: first, to: 'c', branch: 'THEN' },
				{ from: first, to: second, branch: 'ELSE' },
				{ from: second, to: 'c', branch: 'THEN' },
				{ from: second, to: 'd', branch: 'ELSE' },
				{ from: 'c', to: 'd', branch: 'NEXT' },
			]),
		)
	})

	it('derives stable uuid condition ids from the question id and index', () => {
		expect(conditionNodeId('c', 0)).toBe(conditionNodeId('c', 0))
		expect(conditionNodeId('c', 0)).not.toBe(conditionNodeId('c', 1))
		expect(conditionNodeId('c', 0)).toMatch(
			/^[0-9a-f]{8}-[0-9a-f]{4}-8[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
		)
		expect(
			conditionOwners(
				form([CHOICES.a, question('c', 'TEXT', { rule: ALL_TWO })]),
			).get(conditionNodeId('c', 1)),
		).toBe('c')
	})

	it('accepts a linear chain whose condition ids came from elsewhere', () => {
		const graph: EditorGraph = {
			start: 'a',
			nodes: [
				{
					type: 'QUESTION',
					id: 'a',
					kind: 'SINGLE_CHOICE',
					label: 'A',
					required: true,
					options: ['Oui', 'Non'],
				},
				{
					type: 'CONDITION',
					id: 'x',
					question: 'a',
					option: 'Oui',
					checked: true,
				},
				{
					type: 'QUESTION',
					id: 'b',
					kind: 'TEXT',
					label: 'B',
					required: false,
				},
				{ type: 'END', id: 'e', message: 'Fin' },
			],
			edges: [
				{ from: 'a', to: 'x', branch: 'NEXT' },
				{ from: 'x', to: 'b', branch: 'THEN' },
				{ from: 'x', to: 'e', branch: 'ELSE' },
				{ from: 'b', to: 'e', branch: 'NEXT' },
			],
		}
		const parsed = parseGraph(graph)
		expect(parsed.kind).toBe('form')
		if (parsed.kind !== 'form') return
		expect(parsed.form.endMessage).toBe('Fin')
		expect(parsed.form.questions[1].rule).toEqual({
			join: 'ALL',
			conditions: [{ question: 'a', option: 'Oui', checked: true }],
		})
	})

	it('refuses a condition that skips several questions', () => {
		const graph: EditorGraph = {
			start: 'a',
			nodes: [
				{
					type: 'QUESTION',
					id: 'a',
					kind: 'SINGLE_CHOICE',
					label: 'A',
					required: true,
					options: ['Oui', 'Non'],
				},
				{
					type: 'CONDITION',
					id: 'x',
					question: 'a',
					option: 'Oui',
					checked: true,
				},
				{
					type: 'QUESTION',
					id: 'b',
					kind: 'TEXT',
					label: 'B',
					required: false,
				},
				{
					type: 'QUESTION',
					id: 'c',
					kind: 'TEXT',
					label: 'C',
					required: false,
				},
				{ type: 'END', id: 'e', message: null },
			],
			edges: [
				{ from: 'a', to: 'x', branch: 'NEXT' },
				{ from: 'x', to: 'b', branch: 'THEN' },
				{ from: 'x', to: 'e', branch: 'ELSE' },
				{ from: 'b', to: 'c', branch: 'NEXT' },
				{ from: 'c', to: 'e', branch: 'NEXT' },
			],
		}
		expect(parseGraph(graph)).toMatchObject({
			kind: 'advanced',
			questionCount: 3,
		})
	})

	it('refuses a graph with two ends', () => {
		const graph = buildGraph(form([question('a', 'TEXT')]))
		graph.nodes.push({ type: 'END', id: 'other', message: null })
		expect(parseGraph(graph)).toMatchObject({
			kind: 'advanced',
			reason: 'several_ends',
		})
	})
})

function mulberry32(seed: number) {
	let state = seed >>> 0
	return () => {
		state = (state + 0x6d2b79f5) >>> 0
		let t = state
		t = Math.imul(t ^ (t >>> 15), t | 1)
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296
	}
}

const OPTION_POOL = ['A', 'B', 'C', 'D']

function randomForm(random: () => number, seed: number): SurveyForm {
	const pick = <T>(items: readonly T[]): T =>
		items[Math.floor(random() * items.length)]
	const count = Math.floor(random() * 7)
	const questions: FormQuestion[] = []
	let identifierTaken = false
	for (let index = 0; index < count; index += 1) {
		const kind = pick(QUESTION_KINDS).kind
		const id = `q-${seed}-${index}`
		const item = question(id, kind, { required: random() < 0.5 })
		if (kind === 'SINGLE_CHOICE' || kind === 'MULTIPLE_CHOICE') {
			item.options = OPTION_POOL.slice(0, 1 + Math.floor(random() * 3))
		}
		if (kind === 'PHOTO' && random() < 0.5) {
			item.maxPhotos = 1 + Math.floor(random() * 20)
		}
		if (
			!identifierTaken &&
			(kind === 'TEXT' || kind === 'EMAIL' || kind === 'PHONE') &&
			random() < 0.3
		) {
			item.identifier = true
			identifierTaken = true
		}
		const sources = questions.filter(
			(candidate) => candidate.options.length > 0 && candidate.rule === null,
		)
		if (sources.length > 0 && random() < 0.5) {
			const size = 1 + Math.floor(random() * 3)
			const conditions = Array.from({ length: size }, () => {
				const source = pick(sources)
				return {
					question: source.id,
					option: pick(source.options),
					checked: random() < 0.7,
				}
			})
			item.rule = {
				join: size === 1 ? 'ALL' : random() < 0.5 ? 'ALL' : 'ANY',
				conditions,
			}
		}
		questions.push(item)
	}
	return {
		questions,
		endId: `end-${seed}`,
		endMessage: random() < 0.5 ? '' : 'Merci',
	}
}

function randomAnswers(
	random: () => number,
	value: SurveyForm,
): Record<string, unknown> {
	const answers: Record<string, unknown> = {}
	for (const item of value.questions) {
		if (item.kind === 'SINGLE_CHOICE' && random() < 0.8) {
			answers[item.id] = {
				Choice: item.options[Math.floor(random() * item.options.length)],
			}
		}
		if (item.kind === 'MULTIPLE_CHOICE' && random() < 0.8) {
			answers[item.id] = {
				Choices: item.options.filter(() => random() < 0.5),
			}
		}
	}
	return answers
}

function holds(answer: unknown, option: string): boolean {
	if (typeof answer !== 'object' || answer === null) return false
	if ('Choice' in answer) return answer.Choice === option
	if ('Choices' in answer && Array.isArray(answer.Choices)) {
		return answer.Choices.includes(option)
	}
	return false
}

function expectedVisits(
	value: SurveyForm,
	answers: Record<string, unknown>,
): string[] {
	return value.questions
		.filter((item) => {
			if (!item.rule) return true
			const results = item.rule.conditions.map(
				(condition) =>
					holds(answers[condition.question], condition.option) ===
					condition.checked,
			)
			return item.rule.join === 'ALL'
				? results.every(Boolean)
				: results.some(Boolean)
		})
		.map((item) => item.id)
}

describe('random forms', () => {
	it('round trip, validate and follow the expected path for 200 seeded forms', () => {
		const random = mulberry32(20261115)
		for (let seed = 0; seed < 200; seed += 1) {
			const value = randomForm(random, seed)
			const graph = buildGraph(value)
			expect(parseGraph(graph)).toEqual({ kind: 'form', form: value })
			if (value.questions.length === 0) continue
			expect(validateGraph(normalizeGraph(graph))).toEqual([])
			for (let draw = 0; draw < 5; draw += 1) {
				const answers = randomAnswers(random, value)
				expect(answeredPath(graph, answers).map((node) => node.id)).toEqual(
					expectedVisits(value, answers),
				)
			}
		}
	})
})

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
	)('%s passes the validator and keeps its path when representable', (_, item) => {
		const graph = fromSchema(item.graph)
		expect(validateGraph(graph)).toEqual([])
		const parsed = parseGraph(graph)
		if (parsed.kind !== 'form') return
		const rebuilt = buildGraph(parsed.form)
		expect(validateGraph(normalizeGraph(rebuilt))).toEqual([])
		expect(answeredPath(rebuilt, item.answers).map((node) => node.id)).toEqual(
			item.expected_questions,
		)
	})
})
