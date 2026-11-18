import type { Branch, EditorGraph } from '#/pages/survey/types'

export interface SurveyNeighbour {
	id: string
	branch: Branch
}

const BRANCH_ORDER: readonly Branch[] = ['NEXT', 'THEN', 'ELSE']

export function surveyNeighboursOf(
	graph: EditorGraph,
	nodeId: string,
): { previous: SurveyNeighbour[]; next: SurveyNeighbour[] } {
	const order = new Map(graph.nodes.map((node, index) => [node.id, index]))
	const previous = new Map<string, SurveyNeighbour>()
	const next = new Map<string, SurveyNeighbour>()
	for (const edge of graph.edges) {
		if (!order.has(edge.from) || !order.has(edge.to)) continue
		if (edge.to === nodeId && !previous.has(edge.from)) {
			previous.set(edge.from, { id: edge.from, branch: edge.branch })
		}
		if (edge.from === nodeId && !next.has(edge.to)) {
			next.set(edge.to, { id: edge.to, branch: edge.branch })
		}
	}
	return {
		previous: [...previous.values()].sort(
			(a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0),
		),
		next: [...next.values()].sort(
			(a, b) => BRANCH_ORDER.indexOf(a.branch) - BRANCH_ORDER.indexOf(b.branch),
		),
	}
}
