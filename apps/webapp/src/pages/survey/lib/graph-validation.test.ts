import { describe, expect, it } from 'vitest'
import {
	validateGraph,
	validateSentGraph,
} from '#/pages/survey/lib/graph-validation'
import { REASON_MESSAGES } from '#/pages/survey/lib/reason-messages'
import { toSchema } from '#/pages/survey/lib/survey-graph'
import type {
	ConditionNode,
	EditorGraph,
	EndNode,
	GraphEdge,
	QuestionKind,
	QuestionNode,
} from '#/pages/survey/types'

function question(
	id: string,
	kind: QuestionKind = 'TEXT',
	patch: Partial<QuestionNode> = {},
): QuestionNode {
	return {
		type: 'QUESTION',
		id,
		kind,
		label: `Question ${id}`,
		required: false,
		options:
			kind === 'SINGLE_CHOICE' || kind === 'MULTIPLE_CHOICE'
				? ['Oui', 'Non']
				: [],
		...patch,
	}
}

function condition(
	id: string,
	source: string,
	option = 'Oui',
	checked = true,
): ConditionNode {
	return { type: 'CONDITION', id, question: source, option, checked }
}

function end(id: string): EndNode {
	return { type: 'END', id, message: 'Merci' }
}

function next(from: string, to: string): GraphEdge {
	return { from, to, branch: 'NEXT' }
}

function codes(graph: EditorGraph): string[] {
	return validateGraph(graph).map((issue) => issue.code)
}

function linear(): EditorGraph {
	return {
		start: 'q1',
		nodes: [question('q1'), end('e1')],
		edges: [next('q1', 'e1')],
	}
}

function branching(): EditorGraph {
	return {
		start: 'q1',
		nodes: [
			question('q1', 'SINGLE_CHOICE'),
			condition('c1', 'q1'),
			question('q2'),
			question('q3'),
			question('q4'),
			end('e1'),
		],
		edges: [
			next('q1', 'c1'),
			{ from: 'c1', to: 'q2', branch: 'THEN' },
			{ from: 'c1', to: 'q3', branch: 'ELSE' },
			next('q2', 'q4'),
			next('q3', 'q4'),
			next('q4', 'e1'),
		],
	}
}

describe('validateGraph', () => {
	it('accepts a linear graph and a graph whose branches join', () => {
		expect(codes(linear())).toEqual([])
		expect(codes(branching())).toEqual([])
	})

	it('refuses an empty graph', () => {
		expect(codes({ start: null, nodes: [], edges: [] })).toEqual([
			'graph_empty',
		])
	})

	it('refuses a graph without start', () => {
		expect(codes({ ...linear(), start: null })).toContain('graph_unknown_start')
	})

	it('refuses a start that is not a question', () => {
		expect(codes({ ...linear(), start: 'e1' })).toContain(
			'graph_start_not_question',
		)
	})

	it('refuses a graph without end', () => {
		expect(
			codes({ start: 'q1', nodes: [question('q1')], edges: [] }),
		).toContain('graph_without_end')
	})

	it('refuses a question without next', () => {
		const graph = linear()
		const issues = validateGraph({ ...graph, edges: [] })
		expect(issues).toContainEqual(
			expect.objectContaining({ code: 'graph_dead_end', nodeId: 'q1' }),
		)
	})

	it('refuses a condition without its Then or its Else', () => {
		const graph = branching()
		const issues = validateGraph({
			...graph,
			edges: graph.edges.filter((edge) => edge.branch !== 'ELSE'),
		})
		expect(issues).toContainEqual(
			expect.objectContaining({ code: 'graph_dead_end', nodeId: 'c1' }),
		)
	})

	it('refuses two exits on the same branch and a branch the node does not allow', () => {
		const graph = linear()
		expect(
			codes({
				...graph,
				nodes: [...graph.nodes, end('e2')],
				edges: [...graph.edges, next('q1', 'e2')],
			}),
		).toContain('graph_duplicate_exit')
		expect(
			codes({
				...graph,
				edges: [...graph.edges, { from: 'q1', to: 'e1', branch: 'THEN' }],
			}),
		).toContain('graph_invalid_branch')
	})

	it('refuses an edge towards an unknown node', () => {
		expect(
			codes({ ...linear(), edges: [next('q1', 'e1'), next('q1', 'zz')] }),
		).toContain('graph_unknown_edge_node')
	})

	it('refuses a cycle', () => {
		const graph: EditorGraph = {
			start: 'q1',
			nodes: [
				question('q1', 'SINGLE_CHOICE'),
				question('q2'),
				condition('c1', 'q1'),
				end('e1'),
			],
			edges: [
				next('q1', 'q2'),
				next('q2', 'c1'),
				{ from: 'c1', to: 'q2', branch: 'THEN' },
				{ from: 'c1', to: 'e1', branch: 'ELSE' },
			],
		}
		expect(codes(graph)).toContain('graph_cycle')
	})

	it('refuses a node unreachable from the start', () => {
		const graph = linear()
		expect(
			codes({
				...graph,
				nodes: [...graph.nodes, question('q9')],
				edges: [...graph.edges, next('q9', 'e1')],
			}),
		).toContain('graph_unreachable_node')
	})

	it('refuses a condition on a question not asked on every path to it', () => {
		const graph: EditorGraph = {
			start: 'q1',
			nodes: [
				question('q1', 'SINGLE_CHOICE'),
				condition('c1', 'q1'),
				question('q2', 'SINGLE_CHOICE'),
				condition('c2', 'q2'),
				end('e1'),
			],
			edges: [
				next('q1', 'c1'),
				{ from: 'c1', to: 'q2', branch: 'THEN' },
				{ from: 'c1', to: 'c2', branch: 'ELSE' },
				next('q2', 'c2'),
				{ from: 'c2', to: 'e1', branch: 'THEN' },
				{ from: 'c2', to: 'e1', branch: 'ELSE' },
			],
		}
		expect(validateGraph(graph)).toContainEqual(
			expect.objectContaining({
				code: 'condition_question_not_asked',
				nodeId: 'c2',
			}),
		)
	})

	it('refuses a condition on an unknown question, a non choice question or an unknown option', () => {
		const graph = branching()
		const swap = (replacement: ConditionNode) => ({
			...graph,
			nodes: graph.nodes.map((node) => (node.id === 'c1' ? replacement : node)),
		})
		expect(codes(swap(condition('c1', '')))).toContain(
			'condition_unknown_question',
		)
		expect(codes(swap(condition('c1', 'q2')))).toContain(
			'condition_on_non_choice',
		)
		expect(codes(swap(condition('c1', 'q1', 'Peut-être')))).toContain(
			'condition_unknown_option',
		)
	})

	it('refuses a blank label, a blank option, a repeated option and a choice without option', () => {
		const withQuestion = (node: QuestionNode): EditorGraph => ({
			start: 'q1',
			nodes: [node, end('e1')],
			edges: [next('q1', 'e1')],
		})
		expect(
			codes(withQuestion(question('q1', 'TEXT', { label: '  ' }))),
		).toEqual(['graph_question_label_empty'])
		expect(
			codes(
				withQuestion(question('q1', 'SINGLE_CHOICE', { options: ['A', ' '] })),
			),
		).toEqual(['graph_option_empty'])
		expect(
			codes(
				withQuestion(question('q1', 'SINGLE_CHOICE', { options: ['A', 'A'] })),
			),
		).toEqual(['graph_duplicate_option'])
		expect(
			codes(withQuestion(question('q1', 'MULTIPLE_CHOICE', { options: [] }))),
		).toEqual(['graph_choice_without_options'])
	})

	it('allows one identifier on a text, e-mail or phone question only', () => {
		const graph: EditorGraph = {
			start: 'q1',
			nodes: [
				question('q1', 'EMAIL', { identifier: true }),
				question('q2', 'PHONE', { identifier: true }),
				end('e1'),
			],
			edges: [next('q1', 'q2'), next('q2', 'e1')],
		}
		expect(codes(graph)).toEqual(['graph_second_identifier'])
		expect(
			codes({
				start: 'q1',
				nodes: [question('q1', 'NUMBER', { identifier: true }), end('e1')],
				edges: [next('q1', 'e1')],
			}),
		).toEqual(['graph_identifier_on_invalid_kind'])
		expect(
			codes({
				start: 'q1',
				nodes: [question('q1', 'EMAIL', { identifier: true }), end('e1')],
				edges: [next('q1', 'e1')],
			}),
		).toEqual([])
	})

	it('allows a photo limit between 1 and 20 on a photo question only', () => {
		const withQuestion = (node: QuestionNode): EditorGraph => ({
			start: 'q1',
			nodes: [node, end('e1')],
			edges: [next('q1', 'e1')],
		})
		expect(
			codes(withQuestion(question('q1', 'TEXT', { max_photos: 3 }))),
		).toEqual(['graph_max_photos_on_invalid_kind'])
		expect(
			codes(withQuestion(question('q1', 'PHOTO', { max_photos: 0 }))),
		).toEqual(['graph_zero_max_photos'])
		expect(
			codes(withQuestion(question('q1', 'PHOTO', { max_photos: 21 }))),
		).toEqual(['max_photos_out_of_range'])
		expect(
			codes(withQuestion(question('q1', 'PHOTO', { max_photos: 20 }))),
		).toEqual([])
		expect(
			codes(withQuestion(question('q1', 'PHOTO', { max_photos: null }))),
		).toEqual([])
	})

	it('refuses two nodes sharing an id', () => {
		expect(
			codes({
				start: 'q1',
				nodes: [question('q1'), question('q1'), end('e1')],
				edges: [next('q1', 'e1')],
			}),
		).toContain('graph_duplicate_node')
	})

	it('gives every issue a French message', () => {
		for (const issue of validateGraph({ ...branching(), start: null })) {
			expect(issue.message.trim()).not.toBe('')
		}
	})
})

describe('validateSentGraph', () => {
	it('reports options that only differ by spaces as a duplicate', () => {
		const graph: EditorGraph = {
			start: 'q1',
			nodes: [
				question('q1', 'SINGLE_CHOICE', { options: ['Oui', 'Oui '] }),
				end('e1'),
			],
			edges: [next('q1', 'e1')],
		}
		expect(validateSentGraph(graph).map((issue) => issue.code)).toEqual([
			'graph_duplicate_option',
		])
	})

	it('matches a condition option picked from an untrimmed option once normalised', () => {
		const graph = branching()
		const untrimmed: EditorGraph = {
			...graph,
			nodes: graph.nodes.map((node) =>
				node.id === 'q1'
					? question('q1', 'SINGLE_CHOICE', { options: ['Oui ', 'Non'] })
					: node.id === 'c1'
						? condition('c1', 'q1', 'Oui ')
						: node,
			),
		}
		expect(validateSentGraph(untrimmed)).toEqual([])
		const sent = toSchema(untrimmed)
		const sentCondition = sent?.nodes.find((node) => node.id === 'c1')
		expect(sentCondition).toMatchObject({ type: 'CONDITION', option: 'Oui' })
	})
})

describe('validateGraph blocks', () => {
	const B = 'block-1'
	const member = (id: string, kind: QuestionKind = 'TEXT') =>
		question(id, kind, { block: B })

	function blockGraph(): EditorGraph {
		return {
			start: 'q1',
			nodes: [
				question('q1', 'SINGLE_CHOICE'),
				condition('c1', 'q1'),
				member('q2'),
				member('q3'),
				question('q4'),
				end('e1'),
			],
			edges: [
				next('q1', 'c1'),
				{ from: 'c1', to: 'q2', branch: 'THEN' },
				{ from: 'c1', to: 'q4', branch: 'ELSE' },
				next('q2', 'q3'),
				next('q3', 'q4'),
				next('q4', 'e1'),
			],
		}
	}

	it('accepts a block guarded by a rule and treats a null block as absent', () => {
		expect(codes(blockGraph())).toEqual([])
		const graph = linear()
		expect(
			codes({
				...graph,
				nodes: graph.nodes.map((node) =>
					node.type === 'QUESTION' ? { ...node, block: null } : node,
				),
			}),
		).toEqual([])
	})

	it('refuses a block of one question', () => {
		expect(
			validateGraph({
				start: 'q1',
				nodes: [member('q1'), question('q2'), end('e1')],
				edges: [next('q1', 'q2'), next('q2', 'e1')],
			}),
		).toContainEqual(
			expect.objectContaining({ code: 'graph_block_too_small', nodeId: 'q1' }),
		)
	})

	it('refuses a block split by another question', () => {
		expect(
			validateGraph({
				start: 'q1',
				nodes: [member('q1'), question('q2'), member('q3'), end('e1')],
				edges: [next('q1', 'q2'), next('q2', 'q3'), next('q3', 'e1')],
			}),
		).toContainEqual(
			expect.objectContaining({
				code: 'graph_block_not_contiguous',
				nodeId: 'q3',
			}),
		)
	})

	it('refuses a member entered from outside the block', () => {
		const graph = blockGraph()
		expect(
			validateGraph({
				...graph,
				edges: graph.edges.map((edge) =>
					edge.from === 'c1' && edge.branch === 'ELSE'
						? { ...edge, to: 'q3' }
						: edge,
				),
			}),
		).toContainEqual(
			expect.objectContaining({
				code: 'graph_block_not_contiguous',
				nodeId: 'q3',
			}),
		)
	})

	it('refuses a condition between two members', () => {
		const issues = validateGraph({
			start: 'q1',
			nodes: [
				member('q1', 'SINGLE_CHOICE'),
				condition('c1', 'q1'),
				member('q2'),
				end('e1'),
			],
			edges: [
				next('q1', 'c1'),
				{ from: 'c1', to: 'q2', branch: 'THEN' },
				{ from: 'c1', to: 'e1', branch: 'ELSE' },
				next('q2', 'e1'),
			],
		})
		expect(issues).toContainEqual(
			expect.objectContaining({
				code: 'graph_block_member_has_condition',
				nodeId: 'q2',
			}),
		)
		expect(issues.map((item) => item.code)).not.toContain(
			'graph_block_not_contiguous',
		)
	})

	it('refuses a condition whose source is a member of the block it guards', () => {
		const graph = blockGraph()
		expect(
			validateGraph({
				...graph,
				nodes: graph.nodes.map((node) =>
					node.id === 'q1' ? member('q1', 'SINGLE_CHOICE') : node,
				),
			}).map((item) => item.code),
		).toContain('graph_block_member_has_condition')
	})

	it('accepts a condition whose source is a member of an earlier block', () => {
		expect(
			codes({
				start: 'q1',
				nodes: [
					member('q1', 'SINGLE_CHOICE'),
					member('q2'),
					condition('c1', 'q1'),
					question('q3'),
					end('e1'),
				],
				edges: [
					next('q1', 'q2'),
					next('q2', 'c1'),
					{ from: 'c1', to: 'q3', branch: 'THEN' },
					{ from: 'c1', to: 'e1', branch: 'ELSE' },
					next('q3', 'e1'),
				],
			}),
		).toEqual([])
	})

	it('gives the block issues their French message', () => {
		const issue = validateGraph({
			start: 'q1',
			nodes: [member('q1'), end('e1')],
			edges: [next('q1', 'e1')],
		})[0]
		expect(issue?.code).toBe('graph_block_too_small')
		expect(issue?.message).toBe(REASON_MESSAGES.graph_block_too_small)
	})
})
