import {
	isChoiceKind,
	isIdentifierKind,
} from '#/pages/survey/lib/question-kinds'
import { reasonMessage } from '#/pages/survey/lib/reason-messages'
import { normalizeGraph } from '#/pages/survey/lib/survey-graph'
import type {
	Branch,
	EditorGraph,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

export const MAX_PHOTOS_LIMIT = 20

export interface GraphIssue {
	code: string
	nodeId: string | null
	message: string
}

const CLIENT_MESSAGES: Record<string, string> = {
	max_photos_out_of_range: `Le nombre maximal de photos va de 1 à ${MAX_PHOTOS_LIMIT}.`,
}

function issue(code: string, nodeId: string | null): GraphIssue {
	return { code, nodeId, message: CLIENT_MESSAGES[code] ?? reasonMessage(code) }
}

const ALLOWED_BRANCHES: Record<GraphNode['type'], readonly Branch[]> = {
	QUESTION: ['NEXT'],
	CONDITION: ['THEN', 'ELSE'],
	END: [],
}

function questionIssues(node: QuestionNode): GraphIssue[] {
	const issues: GraphIssue[] = []
	const choice = isChoiceKind(node.kind)
	const options = choice ? (node.options ?? []) : []
	if (node.identifier && !isIdentifierKind(node.kind)) {
		issues.push(issue('graph_identifier_on_invalid_kind', node.id))
	}
	if (node.max_photos != null) {
		if (node.kind !== 'PHOTO') {
			issues.push(issue('graph_max_photos_on_invalid_kind', node.id))
		} else if (node.max_photos <= 0) {
			issues.push(issue('graph_zero_max_photos', node.id))
		} else if (node.max_photos > MAX_PHOTOS_LIMIT) {
			issues.push(issue('max_photos_out_of_range', node.id))
		}
	}
	if (choice && options.length === 0) {
		issues.push(issue('graph_choice_without_options', node.id))
	}
	if (node.label.trim() === '') {
		issues.push(issue('graph_question_label_empty', node.id))
	}
	if (options.some((option) => option.trim() === '')) {
		issues.push(issue('graph_option_empty', node.id))
	}
	if (new Set(options).size !== options.length) {
		issues.push(issue('graph_duplicate_option', node.id))
	}
	return issues
}

function identifierIssues(nodes: readonly GraphNode[]): GraphIssue[] {
	const identifiers = nodes.filter(
		(node): node is QuestionNode =>
			node.type === 'QUESTION' &&
			Boolean(node.identifier) &&
			isIdentifierKind(node.kind),
	)
	return identifiers
		.slice(1)
		.map((node) => issue('graph_second_identifier', node.id))
}

export function successorsOf(graph: EditorGraph, id: string): string[] {
	return graph.edges.filter((edge) => edge.from === id).map((edge) => edge.to)
}

function exitIssues(
	graph: EditorGraph,
	ids: ReadonlySet<string>,
): GraphIssue[] {
	const issues: GraphIssue[] = []
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const seen = new Set<string>()
	for (const edge of graph.edges) {
		const from = byId.get(edge.from)
		if (!from || !ids.has(edge.to)) {
			issues.push(issue('graph_unknown_edge_node', from ? edge.to : edge.from))
			continue
		}
		if (!ALLOWED_BRANCHES[from.type].includes(edge.branch)) {
			issues.push(issue('graph_invalid_branch', from.id))
			continue
		}
		const key = `${edge.from}:${edge.branch}`
		if (seen.has(key)) issues.push(issue('graph_duplicate_exit', from.id))
		seen.add(key)
	}
	for (const node of graph.nodes) {
		const missing = ALLOWED_BRANCHES[node.type].some(
			(branch) => !seen.has(`${node.id}:${branch}`),
		)
		if (missing) issues.push(issue('graph_dead_end', node.id))
	}
	return issues
}

function cycleIssues(graph: EditorGraph, start: string): GraphIssue[] {
	const marks = new Map<string, 'open' | 'closed'>()
	const issues: GraphIssue[] = []
	const visit = (id: string) => {
		marks.set(id, 'open')
		for (const successor of successorsOf(graph, id)) {
			const mark = marks.get(successor)
			if (mark === 'open') issues.push(issue('graph_cycle', successor))
			else if (mark === undefined) visit(successor)
		}
		marks.set(id, 'closed')
	}
	visit(start)
	return issues
}

function reachableFrom(graph: EditorGraph, start: string): Set<string> {
	const reached = new Set<string>()
	const stack = [start]
	while (stack.length > 0) {
		const id = stack.pop() as string
		if (reached.has(id)) continue
		reached.add(id)
		stack.push(...successorsOf(graph, id))
	}
	return reached
}

export function topologicalOrder(
	graph: EditorGraph,
	start: string,
): string[] | null {
	const reached = reachableFrom(graph, start)
	const incoming = new Map<string, number>()
	for (const id of reached) incoming.set(id, 0)
	for (const edge of graph.edges) {
		if (reached.has(edge.from) && reached.has(edge.to)) {
			incoming.set(edge.to, (incoming.get(edge.to) ?? 0) + 1)
		}
	}
	const queue = [...reached].filter((id) => incoming.get(id) === 0)
	const order: string[] = []
	while (queue.length > 0) {
		const id = queue.shift() as string
		order.push(id)
		for (const successor of successorsOf(graph, id)) {
			if (!reached.has(successor)) continue
			const remaining = (incoming.get(successor) ?? 0) - 1
			incoming.set(successor, remaining)
			if (remaining === 0) queue.push(successor)
		}
	}
	return order.length === reached.size ? order : null
}

export function askedBefore(
	graph: EditorGraph,
	start: string,
): Map<string, Set<string>> | null {
	const order = topologicalOrder(graph, start)
	if (!order) return null
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const asked = new Map<string, Set<string>>([[start, new Set()]])
	for (const id of order) {
		const before = asked.get(id) ?? new Set<string>()
		const carried = new Set(before)
		if (byId.get(id)?.type === 'QUESTION') carried.add(id)
		for (const successor of successorsOf(graph, id)) {
			const previous = asked.get(successor)
			asked.set(
				successor,
				previous
					? new Set([...previous].filter((item) => carried.has(item)))
					: new Set(carried),
			)
		}
	}
	return asked
}

function conditionIssues(graph: EditorGraph, start: string): GraphIssue[] {
	const asked = askedBefore(graph, start)
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const issues: GraphIssue[] = []
	for (const node of graph.nodes) {
		if (node.type !== 'CONDITION') continue
		const target = byId.get(node.question)
		if (!target) {
			issues.push(issue('condition_unknown_question', node.id))
			continue
		}
		if (target.type !== 'QUESTION' || !isChoiceKind(target.kind)) {
			issues.push(issue('condition_on_non_choice', node.id))
			continue
		}
		if (!(target.options ?? []).includes(node.option)) {
			issues.push(issue('condition_unknown_option', node.id))
			continue
		}
		const before = asked?.get(node.id)
		if (asked && before && !before.has(node.question)) {
			issues.push(issue('condition_question_not_asked', node.id))
		}
	}
	return issues
}

export function validateGraph(graph: EditorGraph): GraphIssue[] {
	if (graph.nodes.length === 0) return [issue('graph_empty', null)]
	const issues: GraphIssue[] = []
	const ids = new Set<string>()
	for (const node of graph.nodes) {
		if (ids.has(node.id)) issues.push(issue('graph_duplicate_node', node.id))
		ids.add(node.id)
	}
	if (!graph.nodes.some((node) => node.type === 'END')) {
		issues.push(issue('graph_without_end', null))
	}
	const startNode = graph.nodes.find((node) => node.id === graph.start)
	if (!startNode) issues.push(issue('graph_unknown_start', null))
	else if (startNode.type !== 'QUESTION') {
		issues.push(issue('graph_start_not_question', null))
	}
	for (const node of graph.nodes) {
		if (node.type === 'QUESTION') issues.push(...questionIssues(node))
	}
	issues.push(...identifierIssues(graph.nodes))
	issues.push(...exitIssues(graph, ids))
	if (startNode) {
		const cycles = cycleIssues(graph, startNode.id)
		issues.push(...cycles)
		const reached = reachableFrom(graph, startNode.id)
		for (const node of graph.nodes) {
			if (!reached.has(node.id)) {
				issues.push(issue('graph_unreachable_node', node.id))
			}
		}
		if (cycles.length === 0)
			issues.push(...conditionIssues(graph, startNode.id))
	} else {
		issues.push(...conditionIssues(graph, ''))
	}
	return issues
}

export function validateSentGraph(graph: EditorGraph): GraphIssue[] {
	return validateGraph(normalizeGraph(graph))
}

export function issuesByNode(
	issues: readonly GraphIssue[],
): Map<string, string[]> {
	const grouped = new Map<string, string[]>()
	for (const item of issues) {
		if (!item.nodeId) continue
		const messages = grouped.get(item.nodeId) ?? []
		if (!messages.includes(item.message)) messages.push(item.message)
		grouped.set(item.nodeId, messages)
	}
	return grouped
}
