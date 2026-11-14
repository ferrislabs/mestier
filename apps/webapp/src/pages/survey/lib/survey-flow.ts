import type { Edge, Node } from '@xyflow/react'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { tidyLayout } from '#/pages/automation/lib/layout'
import {
	type ExitHandle,
	exitsOf,
	questionLabel,
	START_HANDLE,
	START_ID,
} from '#/pages/survey/lib/survey-graph'
import type {
	ConditionNode,
	EditorGraph,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

export const BRANCH_LABELS: Record<ExitHandle, string> = {
	START: 'Départ',
	NEXT: 'Suite',
	THEN: 'Oui',
	ELSE: 'Non',
}

export interface FlowData extends Record<string, unknown> {
	node: GraphNode | null
	issues: string[]
	highlighted: boolean
	readOnly: boolean
	sentence: string
	isStart: boolean
}

export type FlowNodeType = 'start' | 'question' | 'condition' | 'end'

export type SurveyFlowNode = Node<FlowData, FlowNodeType>

const TYPE_OF: Record<GraphNode['type'], FlowNodeType> = {
	QUESTION: 'question',
	CONDITION: 'condition',
	END: 'end',
}

export function conditionSentence(
	graph: EditorGraph,
	condition: ConditionNode,
): string {
	const source = graph.nodes.find(
		(node): node is QuestionNode =>
			node.type === 'QUESTION' && node.id === condition.question,
	)
	const option = condition.option.trim() === '' ? '…' : condition.option
	return `Si « ${questionLabel(source)} » : l’option « ${option} » ${condition.checked ? 'est cochée' : 'n’est pas cochée'}`
}

export function buildFlowNodes({
	graph,
	positions,
	selectedId,
	issues,
	highlightedId,
	readOnly,
	measured,
}: {
	graph: EditorGraph
	positions: ReadonlyMap<string, NodePosition>
	selectedId: string | null
	issues: ReadonlyMap<string, string[]>
	highlightedId: string | null
	readOnly: boolean
	measured: ReadonlyMap<string, { width: number; height: number }>
}): SurveyFlowNode[] {
	const start: SurveyFlowNode = {
		id: START_ID,
		type: 'start',
		position: positions.get(START_ID) ?? { x: 0, y: 0 },
		deletable: false,
		selectable: false,
		measured: measured.get(START_ID),
		data: {
			node: null,
			issues: [],
			highlighted: false,
			readOnly,
			sentence: '',
			isStart: false,
		},
	}
	return [
		start,
		...graph.nodes.map((node) => ({
			id: node.id,
			type: TYPE_OF[node.type],
			position: positions.get(node.id) ?? { x: 0, y: 0 },
			selected: node.id === selectedId,
			deletable: !readOnly,
			measured: measured.get(node.id),
			data: {
				node,
				issues: issues.get(node.id) ?? [],
				highlighted: node.id === highlightedId,
				readOnly,
				sentence:
					node.type === 'CONDITION' ? conditionSentence(graph, node) : '',
				isStart: node.id === graph.start,
			},
		})),
	]
}

export function buildFlowEdges(graph: EditorGraph, readOnly: boolean): Edge[] {
	const ids = new Set(graph.nodes.map((node) => node.id))
	const edges: Edge[] = graph.edges
		.filter((edge) => ids.has(edge.from) && ids.has(edge.to))
		.map((edge) => ({
			id: `${edge.from}:${edge.branch}`,
			source: edge.from,
			sourceHandle: edge.branch,
			target: edge.to,
			deletable: !readOnly,
		}))
	if (graph.start && ids.has(graph.start)) {
		edges.unshift({
			id: `${START_ID}:${START_HANDLE}`,
			source: START_ID,
			sourceHandle: START_HANDLE,
			target: graph.start,
			deletable: !readOnly,
		})
	}
	return edges
}

export function parseEdgeId(
	id: string,
): { from: string; handle: ExitHandle } | null {
	const separator = id.lastIndexOf(':')
	if (separator <= 0) return null
	const handle = id.slice(separator + 1)
	if (
		handle !== 'NEXT' &&
		handle !== 'THEN' &&
		handle !== 'ELSE' &&
		handle !== START_HANDLE
	) {
		return null
	}
	return { from: id.slice(0, separator), handle }
}

function outputsOf(node: GraphNode | null): string[] {
	return node ? exitsOf(node) : [START_HANDLE]
}

export function layoutPositions(
	graph: EditorGraph,
	previous: ReadonlyMap<string, NodePosition>,
): Map<string, NodePosition> {
	const nodes = [
		{ id: START_ID, node: null },
		...graph.nodes.map((node) => ({ id: node.id, node })),
	].map(({ id, node }) => ({
		id,
		position: previous.get(id) ?? { x: 0, y: 0 },
		trigger: node === null,
		outputs: outputsOf(node),
	}))
	return tidyLayout(
		nodes,
		buildFlowEdges(graph, true).map((edge) => ({
			source: edge.source,
			target: edge.target,
			sourceHandle: edge.sourceHandle,
		})),
	)
}
