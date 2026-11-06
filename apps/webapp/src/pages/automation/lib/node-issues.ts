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
import { outputItems } from '#/pages/automation/lib/items'
import {
	LEGACY_SYNTAX_MESSAGE,
	missingKeyMessage,
	notUpstreamMessage,
	unknownNodeMessage,
} from '#/pages/automation/lib/reference-messages'
import { scheduleTimeIssue } from '#/pages/automation/lib/schedule'
import { scheduleOf, webhookOf } from '#/pages/automation/lib/trigger-events'
import {
	displayPath,
	findNodeReferences,
	readPath,
	usesLegacyRoot,
} from '#/pages/automation/lib/variables'
import {
	type ContractKey,
	contractIssues,
} from '#/pages/automation/lib/webhook-contract'
import {
	duplicatePathIssue,
	type PathHolder,
	webhookPathIssue,
} from '#/pages/automation/lib/webhook-path'

export interface NodeIssue {
	parameter: string
	message: string
}

const NO_FORM = { fields: [], copy: {} }

const NO_READS: ReadonlyMap<string, CollectionRead> = new Map()

function stringsOf(value: unknown): string[] {
	if (typeof value === 'string') return [value]
	if (Array.isArray(value)) return value.flatMap(stringsOf)
	if (typeof value === 'object' && value !== null) {
		return Object.values(value).flatMap(stringsOf)
	}
	return []
}

function configEntries(
	connector: NonNullable<EditorNode['data']['connector']>,
): [string, unknown][] {
	return Object.entries(
		connector.kind === COLLECTION_OPERATION_KIND
			? flattenOperationConfig(connector.config, NO_FORM)
			: connector.config,
	)
}

function pathHolders(nodes: readonly EditorNode[]): PathHolder[] {
	return nodes.flatMap((candidate) => {
		const webhook = candidate.data.trigger
			? webhookOf(candidate.data.trigger)
			: null
		return webhook
			? [
					{
						id: candidate.id,
						label: candidate.data.label,
						path: webhook.path,
					},
				]
			: []
	})
}

function keyLabel(key: ContractKey | undefined, index: number): string {
	const name = key?.name.trim()
	return name ? `Clé « ${name} »` : `Clé n° ${index + 1}`
}

function triggerIssues(
	node: EditorNode,
	nodes: readonly EditorNode[],
): NodeIssue[] {
	const trigger = node.data.trigger
	if (!trigger) return []
	const webhook = webhookOf(trigger)
	if (webhook) {
		const pathMessage =
			webhookPathIssue(webhook.path) ??
			duplicatePathIssue(node.id, webhook.path, pathHolders(nodes))
		const contract = webhook.contract ?? []
		return [
			...(pathMessage ? [{ parameter: 'path', message: pathMessage }] : []),
			...contractIssues(contract).flatMap((message, index) =>
				message
					? [
							{
								parameter: 'contract',
								message: `${keyLabel(contract[index], index)} : ${message}`,
							},
						]
					: [],
			),
		]
	}
	const schedule = scheduleOf(trigger)
	const timeMessage = schedule ? scheduleTimeIssue(schedule.time) : null
	return timeMessage ? [{ parameter: 'time', message: timeMessage }] : []
}

export function nodeIssues(
	node: EditorNode,
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	outputOf: (nodeId: string) => unknown,
	reads: ReadonlyMap<string, CollectionRead> = NO_READS,
): NodeIssue[] {
	const connector = node.data.connector
	if (!connector) return triggerIssues(node, nodes)
	const byLabel = new Map<string, EditorNode>()
	for (const candidate of nodes) {
		const label = candidate.data.label.trim()
		if (!byLabel.has(label)) byLabel.set(label, candidate)
	}
	const upstream = new Map(
		upstreamOf(node.id, edges).map((link) => [link.id, link]),
	)
	const issues: NodeIssue[] = []
	const flag = (parameter: string, message: string) => {
		if (
			!issues.some(
				(issue) => issue.parameter === parameter && issue.message === message,
			)
		) {
			issues.push({ parameter, message })
		}
	}
	for (const [parameter, value] of configEntries(connector)) {
		for (const text of stringsOf(value)) {
			if (usesLegacyRoot(text)) flag(parameter, LEGACY_SYNTAX_MESSAGE)
			for (const reference of findNodeReferences(text)) {
				const target = byLabel.get(reference.label.trim())
				const link = target ? upstream.get(target.id) : undefined
				if (!target) {
					flag(parameter, unknownNodeMessage(reference.label))
				} else if (!link) {
					flag(parameter, notUpstreamMessage(reference.label))
				} else if (reference.accessor !== null) {
					const first = outputItems(
						outputOf(target.id),
						link.handle ? [link.handle] : [],
					)[0]
					if (
						first !== undefined &&
						readPath(first, reference.accessor) === undefined
					) {
						flag(
							parameter,
							missingKeyMessage(
								displayPath(reference.accessor),
								reference.label,
							),
						)
					}
				}
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
	outputOf: (nodeId: string) => unknown,
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
