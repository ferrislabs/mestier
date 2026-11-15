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

const BLOCK = 'b0000000-0000-4000-8000-000000000001'
const OTHER_BLOCK = 'b0000000-0000-4000-8000-000000000002'

const BLOCK_TABLE: [string, SurveyForm][] = [
	[
		'an unconditional block',
		form([
			question('x', 'TEXT', { block: BLOCK }),
			question('y', 'NUMBER', { block: BLOCK }),
			question('z', 'DATE', { block: BLOCK }),
		]),
	],
	[
		'a block guarded by an ALL rule',
		form([
			CHOICES.a,
			CHOICES.b,
			question('c', 'TEXT', { rule: ALL_TWO, block: BLOCK }),
			question('d', 'TEXT', { block: BLOCK }),
			question('e', 'TEXT'),
		]),
	],
	[
		'a block guarded by an ANY rule at the end',
		form([
			CHOICES.a,
			CHOICES.b,
			question('c', 'TEXT', {
				rule: { ...ALL_TWO, join: 'ANY' },
				block: BLOCK,
			}),
			question('d', 'PHOTO', { maxPhotos: 2, block: BLOCK }),
		]),
	],
	[
		'two adjacent blocks',
		form([
			question('c', 'TEXT', { block: BLOCK }),
			question('d', 'TEXT', { block: BLOCK }),
			question('e', 'TEXT', { block: OTHER_BLOCK }),
			question('f', 'TEXT', { block: OTHER_BLOCK }),
		]),
	],
	[
		'a rule whose source is a member of an earlier block',
		form([
			{ ...CHOICES.a, block: BLOCK },
			question('c', 'TEXT', { block: BLOCK }),
			question('d', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		]),
	],
]

const DESCRIPTION_TABLE: [string, SurveyForm][] = [
	[
		'a described question',
		form([
			question('a', 'TEXT', { description: 'Tel qu’inscrit sur le contrat' }),
			question('b', 'NUMBER'),
		]),
	],
	[
		'a multi-line description on a guarded question',
		form([
			CHOICES.a,
			question('c', 'LONG_TEXT', {
				description: 'Première ligne\nSeconde ligne',
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		]),
	],
	[
		'described members of a block',
		form([
			question('x', 'TEXT', { block: BLOCK, description: 'Le jour' }),
			question('y', 'PHOTO', { block: BLOCK, maxPhotos: 2 }),
			question('z', 'TEXT', { block: BLOCK, description: 'La ville' }),
		]),
	],
]

describe('form round trip', () => {
	it.each(DESCRIPTION_TABLE)('keeps %s through the graph', (_, value) => {
		const graph = buildGraph(value)
		expect(validateGraph(normalizeGraph(graph))).toEqual([])
		expect(parseGraph(graph)).toEqual({ kind: 'form', form: value })
	})

	it.each(TABLE)('keeps %s through the graph', (_, value) => {
		expect(roundTrip(value)).toEqual({ kind: 'form', form: value })
	})

	it.each(BLOCK_TABLE)('keeps %s through the graph', (_, value) => {
		const graph = buildGraph(value)
		expect(validateGraph(normalizeGraph(graph))).toEqual([])
		expect(parseGraph(graph)).toEqual({ kind: 'form', form: value })
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

	it('writes no block key on a graph without blocks', () => {
		for (const [, value] of TABLE) {
			expect(JSON.stringify(buildGraph(value))).not.toContain('block')
		}
	})

	it('writes no description key on a graph without descriptions', () => {
		for (const [, value] of [...TABLE, ...BLOCK_TABLE]) {
			expect(JSON.stringify(buildGraph(value))).not.toContain('description')
		}
	})

	it('writes a description only when it is not empty', () => {
		const graph = buildGraph(DESCRIPTION_TABLE[0][1])
		expect(
			graph.nodes
				.filter((node) => node.type === 'QUESTION')
				.map((node) => [node.id, node.description]),
		).toEqual([
			['a', 'Tel qu’inscrit sur le contrat'],
			['b', undefined],
		])
	})

	it('reads a null description from the server as empty', () => {
		const value = DESCRIPTION_TABLE[0][1]
		const graph = buildGraph(value)
		const served: EditorGraph = {
			...graph,
			nodes: graph.nodes.map((node) =>
				node.type === 'QUESTION'
					? { ...node, description: node.description ?? null, block: null }
					: node,
			),
		}
		expect(parseGraph(served)).toEqual({ kind: 'form', form: value })
	})

	it('reads a trimmed description from the server as the same form', () => {
		const value = form([question('a', 'TEXT', { description: '  Précisez  ' })])
		const served = normalizeGraph(buildGraph(value))
		expect(parseGraph(served)).toEqual({
			kind: 'form',
			form: form([question('a', 'TEXT', { description: 'Précisez' })]),
		})
	})

	it('reads a null block from the server as no block', () => {
		const value = TABLE[3][1]
		const graph = buildGraph(value)
		const served: EditorGraph = {
			...graph,
			nodes: graph.nodes.map((node) =>
				node.type === 'QUESTION' ? { ...node, block: null } : node,
			),
		}
		expect(parseGraph(served)).toEqual({ kind: 'form', form: value })
	})

	it('chains the members by Next and sends the Else past the last member', () => {
		const graph = buildGraph(BLOCK_TABLE[1][1])
		const first = conditionNodeId('c', 0)
		const second = conditionNodeId('c', 1)
		expect(graph.edges).toEqual(
			expect.arrayContaining([
				{ from: 'b', to: first, branch: 'NEXT' },
				{ from: first, to: second, branch: 'THEN' },
				{ from: first, to: 'e', branch: 'ELSE' },
				{ from: second, to: 'c', branch: 'THEN' },
				{ from: second, to: 'e', branch: 'ELSE' },
				{ from: 'c', to: 'd', branch: 'NEXT' },
				{ from: 'd', to: 'e', branch: 'NEXT' },
			]),
		)
		expect(
			graph.nodes
				.filter((node) => node.type === 'QUESTION')
				.map((node) => [node.id, node.block ?? null]),
		).toEqual([
			['a', null],
			['b', null],
			['c', BLOCK],
			['d', BLOCK],
			['e', null],
		])
	})

	it('refuses a condition guarding a single member of a block', () => {
		const graph = buildGraph(
			form([
				CHOICES.a,
				question('c', 'TEXT', { block: BLOCK }),
				question('d', 'TEXT', { block: BLOCK }),
			]),
		)
		const condition = conditionNodeId('d', 0)
		const rewired: EditorGraph = {
			...graph,
			nodes: [
				...graph.nodes,
				{
					type: 'CONDITION',
					id: condition,
					question: 'a',
					option: 'Oui',
					checked: true,
				},
			],
			edges: [
				...graph.edges.filter((edge) => edge.from !== 'c'),
				{ from: 'c', to: condition, branch: 'NEXT' },
				{ from: condition, to: 'd', branch: 'THEN' },
				{ from: condition, to: END, branch: 'ELSE' },
			],
		}
		expect(parseGraph(rewired)).toMatchObject({
			kind: 'advanced',
			reason: 'shape',
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

function governing(
	questions: readonly FormQuestion[],
	index: number,
): FormQuestion | undefined {
	let first = index
	const block = questions[index]?.block ?? null
	while (block !== null && first > 0 && questions[first - 1]?.block === block) {
		first -= 1
	}
	return questions[first]
}

function randomForm(random: () => number, seed: number): SurveyForm {
	const pick = <T>(items: readonly T[]): T =>
		items[Math.floor(random() * items.length)]
	const count = Math.floor(random() * 9)
	const questions: FormQuestion[] = []
	let identifierTaken = false
	let blocks = 0
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
		const previous = questions[index - 1]
		if (previous && random() < 0.35) {
			if (previous.block === null) {
				blocks += 1
				previous.block = `block-${seed}-${blocks}`
			}
			item.block = previous.block
			questions.push(item)
			continue
		}
		const sources = questions.filter(
			(candidate, at) =>
				candidate.options.length > 0 && governing(questions, at)?.rule === null,
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
	const result: SurveyForm = {
		questions,
		endId: `end-${seed}`,
		endMessage: random() < 0.5 ? '' : 'Merci',
	}
	describeRandomly(result, mulberry32(seed ^ 0x5bd1e995))
	return result
}

const DESCRIPTION_POOL = [
	'Précisez si besoin',
	'Une ligne\nPuis une autre',
	'  Avec des espaces autour  ',
	'é'.repeat(500),
]

function describeRandomly(value: SurveyForm, random: () => number) {
	for (const item of value.questions) {
		if (random() < 0.4) {
			item.description =
				DESCRIPTION_POOL[Math.floor(random() * DESCRIPTION_POOL.length)]
		}
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
		.filter((_, index) => {
			const item = governing(value.questions, index)
			if (!item?.rule) return true
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

function caseSeed(index: number): number {
	return (20261115 + Math.imul(index, 2654435761)) >>> 0
}

const BRANCH_ORDER = ['NEXT', 'THEN', 'ELSE'] as const

function canonical(graph: EditorGraph): string {
	const normalized = normalizeGraph(graph)
	const names = new Map<string, string>()
	const visit = (id: string | null | undefined) => {
		if (!id || names.has(id)) return
		const node = normalized.nodes.find((item) => item.id === id)
		if (!node) return
		names.set(id, node.type === 'CONDITION' ? `c${names.size}` : id)
		for (const branch of BRANCH_ORDER) {
			visit(
				normalized.edges.find(
					(edge) => edge.from === id && edge.branch === branch,
				)?.to,
			)
		}
	}
	visit(normalized.start)
	const rename = (id: string) => names.get(id) ?? `unreachable:${id}`
	return JSON.stringify({
		start: normalized.start ? rename(normalized.start) : null,
		nodes: normalized.nodes
			.map((node) => ({ ...node, id: rename(node.id) }))
			.sort((a, b) => a.id.localeCompare(b.id)),
		edges: normalized.edges
			.map((edge) => `${rename(edge.from)}:${edge.branch}:${rename(edge.to)}`)
			.sort(),
	})
}

describe('random forms', () => {
	it('round trip, validate and follow the expected path for 200 seeded forms', () => {
		let withBlocks = 0
		let withDescriptions = 0
		for (let index = 0; index < 200; index += 1) {
			const seed = caseSeed(index)
			const random = mulberry32(seed)
			const value = randomForm(random, seed)
			const graph = buildGraph(value)
			expect(parseGraph(graph), `round trip, seed ${seed}`).toEqual({
				kind: 'form',
				form: value,
			})
			if (value.questions.length === 0) continue
			if (value.questions.some((item) => item.block !== null)) withBlocks += 1
			if (value.questions.some((item) => item.description !== '')) {
				withDescriptions += 1
			}
			expect(
				validateGraph(normalizeGraph(graph)),
				`validation, seed ${seed}`,
			).toEqual([])
			for (let draw = 0; draw < 5; draw += 1) {
				const answers = randomAnswers(random, value)
				expect(
					answeredPath(graph, answers).map((node) => node.id),
					`path ${draw}, seed ${seed}`,
				).toEqual(expectedVisits(value, answers))
			}
		}
		expect(withBlocks).toBeGreaterThan(50)
		expect(withDescriptions).toBeGreaterThan(50)
	})

	it('reads a randomly rewired graph as advanced or as a form that rebuilds it, for 100 seeded cases', () => {
		let mutated = 0
		for (let index = 0; index < 100; index += 1) {
			const seed = caseSeed(1000 + index)
			const random = mulberry32(seed)
			let value = randomForm(random, seed)
			while (value.questions.length === 0) value = randomForm(random, seed)
			const graph = buildGraph(value)
			const edge = Math.floor(random() * graph.edges.length)
			const target =
				graph.nodes[Math.floor(random() * graph.nodes.length)]?.id ?? ''
			const rewired: EditorGraph = {
				...graph,
				edges: graph.edges.map((item, at) =>
					at === edge ? { ...item, to: target } : item,
				),
			}
			if (graph.edges[edge]?.to !== target) mutated += 1
			const parsed = parseGraph(rewired)
			if (parsed.kind === 'advanced') continue
			expect(
				canonical(buildGraph(parsed.form)),
				`rewired edge ${edge} to ${target}, seed ${seed}`,
			).toBe(canonical(rewired))
		}
		expect(mutated).toBeGreaterThan(50)
	})
})

interface PathCase {
	name: string
	graph: Schemas.SurveyGraphSchema
	answers: Record<string, unknown>
	expected_questions: string[]
	expected_descriptions?: Record<string, string | null>
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
	it('reads every shared block case as a form that keeps its blocks', () => {
		const blocks = CASES.filter((item) =>
			item.graph.nodes.some(
				(node) => node.type === 'QUESTION' && node.block != null,
			),
		)
		expect(blocks).toHaveLength(3)
		for (const item of blocks) {
			const parsed = parseGraph(fromSchema(item.graph))
			expect(parsed.kind, item.name).toBe('form')
			if (parsed.kind !== 'form') continue
			expect(
				parsed.form.questions.filter((question) => question.block !== null)
					.length,
			).toBeGreaterThanOrEqual(2)
		}
	})

	it('reads every shared description case as a form that keeps its trimmed description', () => {
		const described = CASES.filter((item) => item.expected_descriptions)
		expect(described).toHaveLength(2)
		for (const item of described) {
			const parsed = parseGraph(normalizeGraph(fromSchema(item.graph)))
			expect(parsed.kind, item.name).toBe('form')
			if (parsed.kind !== 'form') continue
			for (const [id, expected] of Object.entries(
				item.expected_descriptions ?? {},
			)) {
				expect(
					parsed.form.questions.find((question) => question.id === id)
						?.description,
					item.name,
				).toBe(expected ?? '')
			}
		}
	})

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
