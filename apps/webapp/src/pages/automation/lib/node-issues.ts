import {
	COLLECTION_OPERATION_KIND,
	type CollectionRead,
	flattenOperationConfig,
	operationIssues,
} from '#/pages/automation/lib/collection-node'
import type {
	EditorEdge,
	EditorNode,
} from '#/pages/automation/lib/editor-nodes'
import { upstreamOf } from '#/pages/automation/lib/graph'
import {
	displayPath,
	findConnectorReferences,
	readPath,
} from '#/pages/automation/lib/variables'

export interface NodeIssue {
	parameter: string
	message: string
}

function labelOf(nodes: readonly EditorNode[], id: string): string {
	return nodes.find((node) => node.id === id)?.data.label ?? id
}

const NO_FORM = { fields: [], copy: {} }

const NO_READS: ReadonlyMap<string, CollectionRead> = new Map()

function configEntries(
	connector: NonNullable<EditorNode['data']['connector']>,
): [string, unknown][] {
	return Object.entries(
		connector.kind === COLLECTION_OPERATION_KIND
			? flattenOperationConfig(connector.config, NO_FORM)
			: connector.config,
	)
}

export function nodeIssues(
	node: EditorNode,
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	outputOf: (connectorId: string) => unknown,
	reads: ReadonlyMap<string, CollectionRead> = NO_READS,
): NodeIssue[] {
	const connector = node.data.connector
	if (!connector) return []
	const known = new Set(nodes.map((candidate) => candidate.id))
	const upstream = new Set(upstreamOf(node.id, edges).map((link) => link.id))
	const issues: NodeIssue[] = []
	for (const [parameter, value] of configEntries(connector)) {
		const text = typeof value === 'string' ? value : JSON.stringify(value)
		if (typeof text !== 'string') continue
		for (const reference of findConnectorReferences(text)) {
			const label = labelOf(nodes, reference.connectorId)
			const path = displayPath(reference.accessor)
			const message = !known.has(reference.connectorId)
				? `Le nœud « ${label} » n’existe plus.`
				: !upstream.has(reference.connectorId)
					? `Le nœud « ${label} » n’est pas relié en amont de ce nœud.`
					: readPath(outputOf(reference.connectorId), reference.accessor) ===
								undefined && outputOf(reference.connectorId) !== undefined
						? `La clé « ${path} » est absente de la sortie de « ${label} ».`
						: null
			if (
				message &&
				!issues.some(
					(issue) => issue.parameter === parameter && issue.message === message,
				)
			) {
				issues.push({ parameter, message })
			}
		}
	}
	if (connector.kind === COLLECTION_OPERATION_KIND) {
		issues.push(...operationIssues(connector, reads))
	}
	return issues
}

export function nodeIssuesByNode(
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	outputOf: (connectorId: string) => unknown,
	reads: ReadonlyMap<string, CollectionRead> = NO_READS,
): Map<string, NodeIssue[]> {
	return new Map(
		nodes
			.map(
				(node) =>
					[node.id, nodeIssues(node, nodes, edges, outputOf, reads)] as const,
			)
			.filter(([, issues]) => issues.length > 0),
	)
}
