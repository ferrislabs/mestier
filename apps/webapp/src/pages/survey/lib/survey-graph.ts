import type { Schemas } from '#/api/api.client'
import { isChoiceKind } from '#/pages/survey/lib/question-kinds'
import type {
	Branch,
	EditorGraph,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

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
			...(node.block ? { block: node.block } : {}),
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
