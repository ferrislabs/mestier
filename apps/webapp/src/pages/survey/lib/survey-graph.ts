import type { Schemas } from '#/api/api.client'
import { isChoiceKind } from '#/pages/survey/lib/question-kinds'
import type {
	Branch,
	EditorGraph,
	GraphEdge,
	GraphNode,
	NodeChoice,
	QuestionNode,
} from '#/pages/survey/types'

export const START_ID = '__start__'
export const START_HANDLE = 'START'

export type ExitHandle = Branch | typeof START_HANDLE

export function emptyGraph(): EditorGraph {
	return { start: null, nodes: [], edges: [] }
}

export function fromSchema(
	graph: Schemas.SurveyGraphSchema | null | undefined,
): EditorGraph {
	if (!graph) return emptyGraph()
	return {
		start: graph.start,
		nodes: graph.nodes.map((node) => ({ ...node })),
		edges: graph.edges.map((edge) => ({ ...edge })),
	}
}

function cleanNode(node: GraphNode): GraphNode {
	if (node.type === 'QUESTION') {
		return {
			type: 'QUESTION',
			id: node.id,
			kind: node.kind,
			label: node.label.trim(),
			required: node.required,
			options: isChoiceKind(node.kind)
				? (node.options ?? []).map((option) => option.trim())
				: [],
			max_photos: node.kind === 'PHOTO' ? (node.max_photos ?? null) : null,
			identifier: Boolean(node.identifier),
		}
	}
	if (node.type === 'END') {
		const message = node.message?.trim() ?? ''
		return {
			type: 'END',
			id: node.id,
			message: message === '' ? null : message,
		}
	}
	return { ...node, option: node.option.trim() }
}

export function normalizeGraph(graph: EditorGraph): EditorGraph {
	return {
		start: graph.start,
		nodes: graph.nodes.map(cleanNode),
		edges: graph.edges.map((edge) => ({ ...edge })),
	}
}

export function toSchema(graph: EditorGraph): Schemas.SurveyGraphSchema | null {
	if (graph.nodes.length === 0 || !graph.start) return null
	const normalized = normalizeGraph(graph)
	return {
		start: graph.start,
		nodes: normalized.nodes,
		edges: normalized.edges,
	}
}

export function createNode(choice: NodeChoice, id: string): GraphNode {
	if (choice === 'CONDITION') {
		return { type: 'CONDITION', id, question: '', option: '', checked: true }
	}
	if (choice === 'END') return { type: 'END', id, message: null }
	return {
		type: 'QUESTION',
		id,
		kind: choice,
		label: '',
		required: false,
		options: isChoiceKind(choice) ? ['Option 1', 'Option 2'] : [],
		max_photos: null,
		identifier: false,
	}
}

export function exitsOf(node: GraphNode): Branch[] {
	if (node.type === 'QUESTION') return ['NEXT']
	if (node.type === 'CONDITION') return ['THEN', 'ELSE']
	return []
}

export function openExit(graph: EditorGraph, id: string): ExitHandle | null {
	if (id === START_ID) return graph.start ? null : START_HANDLE
	const node = graph.nodes.find((candidate) => candidate.id === id)
	if (!node) return null
	return (
		exitsOf(node).find(
			(branch) =>
				!graph.edges.some((edge) => edge.from === id && edge.branch === branch),
		) ?? null
	)
}

export function connect(
	graph: EditorGraph,
	from: string,
	handle: ExitHandle,
	to: string,
): EditorGraph {
	if (from === to || to === START_ID) return graph
	if (from === START_ID) return { ...graph, start: to }
	if (handle === START_HANDLE) return graph
	const node = graph.nodes.find((candidate) => candidate.id === from)
	if (!node || !exitsOf(node).includes(handle)) return graph
	const edges: GraphEdge[] = [
		...graph.edges.filter(
			(edge) => !(edge.from === from && edge.branch === handle),
		),
		{ from, to, branch: handle },
	]
	return { ...graph, edges }
}

export function disconnect(
	graph: EditorGraph,
	from: string,
	handle: ExitHandle,
): EditorGraph {
	if (from === START_ID) return { ...graph, start: null }
	return {
		...graph,
		edges: graph.edges.filter(
			(edge) => !(edge.from === from && edge.branch === handle),
		),
	}
}

export function addNode(
	graph: EditorGraph,
	node: GraphNode,
	after: { from: string; handle: ExitHandle } | null,
): EditorGraph {
	const added: EditorGraph = { ...graph, nodes: [...graph.nodes, node] }
	if (after) return connect(added, after.from, after.handle, node.id)
	if (!graph.start && node.type === 'QUESTION') {
		return { ...added, start: node.id }
	}
	return added
}

export function removeNodes(
	graph: EditorGraph,
	ids: ReadonlySet<string>,
): EditorGraph {
	return {
		start: graph.start && ids.has(graph.start) ? null : graph.start,
		nodes: graph.nodes.filter((node) => !ids.has(node.id)),
		edges: graph.edges.filter(
			(edge) => !ids.has(edge.from) && !ids.has(edge.to),
		),
	}
}

export function replaceNode(graph: EditorGraph, node: GraphNode): EditorGraph {
	return {
		...graph,
		nodes: graph.nodes.map((candidate) =>
			candidate.id === node.id ? node : candidate,
		),
	}
}

function ancestorsOf(graph: EditorGraph, id: string): Set<string> {
	const found = new Set<string>()
	const stack = [id]
	while (stack.length > 0) {
		const current = stack.pop() as string
		for (const edge of graph.edges) {
			if (edge.to === current && !found.has(edge.from)) {
				found.add(edge.from)
				stack.push(edge.from)
			}
		}
	}
	return found
}

export function choiceQuestions(graph: EditorGraph): QuestionNode[] {
	return graph.nodes.filter(
		(node): node is QuestionNode =>
			node.type === 'QUESTION' && isChoiceKind(node.kind),
	)
}

export function precedingChoiceQuestions(
	graph: EditorGraph,
	conditionId: string,
): QuestionNode[] {
	const ancestors = ancestorsOf(graph, conditionId)
	const preceding = choiceQuestions(graph).filter((node) =>
		ancestors.has(node.id),
	)
	return preceding.length > 0 ? preceding : choiceQuestions(graph)
}

export function questionLabel(node: QuestionNode | undefined): string {
	if (!node) return 'question à choisir'
	return node.label.trim() === '' ? 'Question sans libellé' : node.label
}

function choiceHolds(answer: unknown, option: string): boolean {
	if (typeof answer !== 'object' || answer === null) return false
	if ('Choice' in answer)
		return (answer as { Choice: unknown }).Choice === option
	if ('Choices' in answer) {
		const choices = (answer as { Choices: unknown }).Choices
		return Array.isArray(choices) && choices.includes(option)
	}
	return false
}

export function answeredPath(
	graph: EditorGraph,
	answers: Readonly<Record<string, unknown>>,
): QuestionNode[] {
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const path: QuestionNode[] = []
	const visited = new Set<string>()
	let current = graph.start
	while (current && !visited.has(current)) {
		visited.add(current)
		const node = byId.get(current)
		if (!node || node.type === 'END') break
		const id: string = node.id
		const branch: Branch =
			node.type === 'QUESTION'
				? 'NEXT'
				: choiceHolds(answers[node.question], node.option) === node.checked
					? 'THEN'
					: 'ELSE'
		if (node.type === 'QUESTION') path.push(node)
		current =
			graph.edges.find((edge) => edge.from === id && edge.branch === branch)
				?.to ?? null
	}
	return path
}

export function identifierQuestion(
	graph: EditorGraph,
): QuestionNode | undefined {
	return graph.nodes.find(
		(node): node is QuestionNode =>
			node.type === 'QUESTION' && Boolean(node.identifier),
	)
}
