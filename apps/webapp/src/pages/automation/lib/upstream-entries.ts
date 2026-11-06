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
import { toItems } from '#/pages/automation/lib/items'
import {
	triggerExample,
	triggerKindOf,
} from '#/pages/automation/lib/trigger-events'

export interface UpstreamEntry {
	id: string
	label: string
	icon: LucideIcon
	hue: number
	rootPath: string
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

function triggerEntry(
	triggers: readonly EditorNode[],
	distance: number,
	handle: string | null,
	sources: UpstreamSources,
): UpstreamEntry | null {
	const first = triggers[0]
	if (!first?.data.trigger) return null
	const example = firstTriggerExample(triggers, sources.events)
	const appearance = triggerAppearance(triggerKindOf(first.data.trigger.kind))
	const payload = sources.lastRun ? sources.lastRun.triggerPayload : example
	return {
		id: 'trigger',
		label: first.data.label,
		icon: appearance.icon,
		hue: appearance.hue,
		rootPath: 'trigger',
		distance,
		direct: distance === 1,
		branch: handle ? branchLabel(handle) : undefined,
		items: payload === undefined || payload === null ? [] : toItems(payload),
		executed: sources.lastRun !== null,
		status: sources.lastRun ? 'success' : null,
		error: null,
	}
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

export function upstreamEntries(
	nodeId: string,
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
	sources: UpstreamSources,
): UpstreamEntry[] {
	const byId = new Map(nodes.map((node) => [node.id, node]))
	const links = upstreamOf(nodeId, edges)
	const triggerLink = links.find((link) => byId.get(link.id)?.data.trigger)
	const triggers = triggersLinkedFirst(nodes, byId, links)
	const farthest = links.reduce((max, link) => Math.max(max, link.distance), 0)

	const entries: UpstreamEntry[] = []
	if (byId.get(nodeId)?.data.kind === 'connector') {
		const entry = triggerEntry(
			triggers,
			triggerLink?.distance ?? farthest + 1,
			triggerLink?.handle ?? null,
			sources,
		)
		if (entry) entries.push(entry)
	}
	for (const link of links) {
		const source = byId.get(link.id)
		if (!source?.data.connector) continue
		const appearance = connectorAppearance(
			source.data.family,
			source.data.connector?.kind,
		)
		const real = sources.lastRun?.connectorOutputs[source.id]
		const output = real ?? connectorOutputExample(source, sources.descriptors)
		entries.push({
			id: source.id,
			label: source.data.label,
			icon: appearance.icon,
			hue: appearance.hue,
			rootPath: `connectors.${source.id}.output`,
			distance: link.distance,
			direct: link.distance === 1,
			branch: link.handle ? branchLabel(link.handle) : undefined,
			items: output === undefined || output === null ? [] : toItems(output),
			executed: real !== undefined,
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
	const connectors = links.flatMap((link) => {
		const source = byId.get(link.id)
		return source?.data.connector
			? [
					{
						id: source.id,
						label: source.data.label,
						output: connectorOutput(source, sources),
					},
				]
			: []
	})
	return toEvaluateContext({ trigger, connectors })
}
