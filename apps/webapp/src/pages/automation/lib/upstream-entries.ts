import type { LucideIcon } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { branchLabel } from '#/pages/automation/lib/branch-labels'
import {
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import { toEvaluateContext } from '#/pages/automation/lib/data-tree'
import type {
	EditorEdge,
	EditorNode,
} from '#/pages/automation/lib/editor-nodes'
import { upstreamOf } from '#/pages/automation/lib/graph'
import { outputItems, toItems, triggerItem } from '#/pages/automation/lib/items'
import {
	triggerExample,
	triggerKindOf,
} from '#/pages/automation/lib/trigger-events'

export interface UpstreamEntry {
	id: string
	label: string
	icon: LucideIcon
	hue: number
	distance: number
	direct: boolean
	branch?: string
	items: readonly unknown[]
	executed: boolean
	status: 'success' | 'error' | null
	error: string | null
}

export interface LastRunData {
	triggerPayload: unknown
	connectorOutputs: Record<string, unknown>
}

export interface UpstreamSources {
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	events: readonly Schemas.EventDescriptorResponse[]
	lastRun: LastRunData | null
	runSteps: Map<string, Schemas.RunStepResponse>
}

export function connectorOutputExample(
	node: EditorNode,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
): unknown {
	const connector = node.data.connector
	if (!connector) return null
	const descriptor = descriptors.get(connector.kind)
	const mirrored = descriptor?.output_mirrors_field
	if (mirrored) return connector.config[mirrored] ?? {}
	return descriptor?.output_example ?? null
}

export function connectorOutput(
	node: EditorNode,
	sources: UpstreamSources,
): unknown {
	return (
		sources.lastRun?.connectorOutputs[node.id] ??
		connectorOutputExample(node, sources.descriptors)
	)
}

export function nodeOutput(
	node: EditorNode,
	sources: UpstreamSources,
): unknown {
	if (!node.data.trigger) return connectorOutput(node, sources)
	return sources.lastRun
		? sources.lastRun.triggerPayload
		: triggerExample(node.data.trigger, sources.events)
}

function stepOutcome(
	step: Schemas.RunStepResponse | undefined,
): Pick<UpstreamEntry, 'status' | 'error'> {
	if (!step) return { status: null, error: null }
	if (step.error) return { status: 'error', error: step.error }
	return { status: step.output === undefined ? null : 'success', error: null }
}

function firstTriggerExample(
	triggers: readonly EditorNode[],
	events: readonly Schemas.EventDescriptorResponse[],
): unknown {
	for (const node of triggers) {
		const example = node.data.trigger
			? triggerExample(node.data.trigger, events)
			: undefined
		if (example !== undefined) return example
	}
	return undefined
}

function triggerItems(node: EditorNode, sources: UpstreamSources): unknown[] {
	const payload = nodeOutput(node, sources)
	if (sources.lastRun) return [triggerItem(payload)]
	return payload === undefined || payload === null ? [] : [triggerItem(payload)]
}

function triggersLinkedFirst(
	nodes: readonly EditorNode[],
	byId: ReadonlyMap<string, EditorNode>,
	links: readonly { id: string }[],
): EditorNode[] {
	const linked = links.flatMap((link) => {
		const node = byId.get(link.id)
		return node?.data.trigger ? [node] : []
	})[0]
	return [
		...(linked ? [linked] : []),
		...nodes.filter(
			(node) => node.data.kind === 'trigger' && node.id !== linked?.id,
		),
	]
}

function connectorItems(
	node: EditorNode,
	handle: string | null,
	sources: UpstreamSources,
): unknown[] {
	const real = sources.lastRun?.connectorOutputs[node.id]
	if (real !== undefined) return outputItems(real, handle ? [handle] : [])
	const example = connectorOutputExample(node, sources.descriptors)
	return example === undefined || example === null ? [] : toItems(example)
}

export function upstreamEntries(
	nodeId: string,
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	sources: UpstreamSources,
): UpstreamEntry[] {
	const byId = new Map(nodes.map((node) => [node.id, node]))
	const entries: UpstreamEntry[] = []
	for (const link of upstreamOf(nodeId, edges)) {
		const source = byId.get(link.id)
		if (!source) continue
		const branch = link.handle ? branchLabel(link.handle) : undefined
		if (source.data.trigger) {
			if (byId.get(nodeId)?.data.kind !== 'connector') continue
			const appearance = triggerAppearance(
				triggerKindOf(source.data.trigger.kind),
			)
			entries.push({
				id: source.id,
				label: source.data.label,
				icon: appearance.icon,
				hue: appearance.hue,
				distance: link.distance,
				direct: link.distance === 1,
				branch,
				items: triggerItems(source, sources),
				executed: sources.lastRun !== null,
				status: sources.lastRun ? 'success' : null,
				error: null,
			})
			continue
		}
		if (!source.data.connector) continue
		const appearance = connectorAppearance(
			source.data.family,
			source.data.connector.kind,
		)
		entries.push({
			id: source.id,
			label: source.data.label,
			icon: appearance.icon,
			hue: appearance.hue,
			distance: link.distance,
			direct: link.distance === 1,
			branch,
			items: connectorItems(source, link.handle, sources),
			executed: sources.lastRun?.connectorOutputs[source.id] !== undefined,
			...stepOutcome(sources.runSteps.get(source.id)),
		})
	}
	return entries.sort((a, b) => a.distance - b.distance)
}

export function evaluateContextFor(
	nodeId: string,
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	sources: UpstreamSources,
): Schemas.EvaluateContextBody {
	const byId = new Map(nodes.map((node) => [node.id, node]))
	const links = upstreamOf(nodeId, edges)
	const trigger = sources.lastRun
		? sources.lastRun.triggerPayload
		: firstTriggerExample(
				triggersLinkedFirst(nodes, byId, links),
				sources.events,
			)
	const available = links.flatMap((link) => {
		const source = byId.get(link.id)
		if (!source) return []
		if (source.data.trigger) {
			return [
				{ label: source.data.label, items: triggerItems(source, sources) },
			]
		}
		return source.data.connector
			? [
					{
						label: source.data.label,
						items: connectorItems(source, link.handle, sources),
					},
				]
			: []
	})
	return toEvaluateContext({ trigger, nodes: available })
}
