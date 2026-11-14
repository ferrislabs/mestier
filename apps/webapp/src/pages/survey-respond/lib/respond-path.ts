import { answeredPath } from '#/pages/survey/lib/survey-graph'
import type {
	Branch,
	EditorGraph,
	EndNode,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

export interface RespondPath {
	questions: QuestionNode[]
	end: EndNode | null
}

function choiceHolds(answer: unknown, option: string): boolean {
	if (typeof answer !== 'object' || answer === null) return false
	if ('Choice' in answer) return answer.Choice === option
	if ('Choices' in answer) {
		return Array.isArray(answer.Choices) && answer.Choices.includes(option)
	}
	return false
}

function nextOf(
	graph: EditorGraph,
	id: string,
	branch: Branch,
): string | undefined {
	return graph.edges.find((edge) => edge.from === id && edge.branch === branch)
		?.to
}

function branchOf(
	node: GraphNode,
	answers: Readonly<Record<string, unknown>>,
): Branch {
	if (node.type !== 'CONDITION') return 'NEXT'
	return choiceHolds(answers[node.question], node.option) === node.checked
		? 'THEN'
		: 'ELSE'
}

export function respondPath(
	graph: EditorGraph,
	answers: Readonly<Record<string, unknown>>,
): RespondPath {
	const questions = answeredPath(graph, answers)
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const visited = new Set<string>()
	let current = questions.at(-1)?.id ?? graph.start ?? undefined
	while (current && !visited.has(current)) {
		visited.add(current)
		const node = byId.get(current)
		if (!node) break
		if (node.type === 'END') return { questions, end: node }
		current = nextOf(graph, node.id, branchOf(node, answers))
	}
	return { questions, end: null }
}

export function knownLength(
	graph: EditorGraph,
	questionId: string,
	pathLength: number,
): number | null {
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const visited = new Set<string>()
	let current = nextOf(graph, questionId, 'NEXT')
	while (current && !visited.has(current)) {
		visited.add(current)
		const node = byId.get(current)
		if (!node || node.type === 'END') return pathLength
		if (node.type === 'CONDITION') return null
		current = nextOf(graph, node.id, 'NEXT')
	}
	return pathLength
}
