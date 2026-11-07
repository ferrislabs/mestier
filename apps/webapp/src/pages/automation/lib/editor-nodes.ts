import type { Edge, Node } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import {
	connectorCopy,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import type { NodePosition } from '#/pages/automation/lib/graph'
import {
	referencedLabels,
	renameReferencesInValue,
} from '#/pages/automation/lib/references'
import {
	triggerKindOf,
	triggerPresentation,
} from '#/pages/automation/lib/trigger-events'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'

export interface NodeDetailsPatch {
	name?: string
	description?: string
}

export interface EditorLookup {
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	events: readonly Schemas.EventDescriptorResponse[]
}

export interface EditorNodeData extends Record<string, unknown> {
	kind: 'connector' | 'trigger'
	label: string
	subtitle: string
	disabled: boolean
	family?: string
	typeLabel: string
	branches: Schemas.BranchDto[]
	errors: ConnectorValidationError[]
	runStatus: string | null
	connector?: Schemas.PlacedConnectorDto
	trigger?: Schemas.PlacedTriggerDto
}

export type EditorNode = Node<EditorNodeData, 'automation'>
export type EditorEdge = Edge

const UNKNOWN_CONNECTOR_SUBTITLE = 'Connecteur'

export function edgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

export function edgeIdOf(
	source: string,
	target: string,
	handle: string | null | undefined,
): string {
	return `${source}->${target}:${handle ?? ''}`
}

function defaultPresentation(
	data: EditorNodeData,
	lookup: EditorLookup,
): { label: string; subtitle: string; family?: string; typeLabel: string } {
	if (data.trigger) {
		return {
			...triggerPresentation(data.trigger, lookup.events),
			typeLabel: triggerCopy(triggerKindOf(data.trigger.kind)).label,
		}
	}
	const connector = data.connector
	const descriptor = connector
		? lookup.descriptors.get(connector.kind)
		: undefined
	const copy = connector ? connectorCopy(connector.kind) : undefined
	return {
		label: copy?.label ?? descriptor?.label ?? connector?.id ?? '',
		subtitle: copy?.subtitle ?? UNKNOWN_CONNECTOR_SUBTITLE,
		family: descriptor?.family,
		typeLabel: copy?.label ?? descriptor?.label ?? connector?.kind ?? '',
	}
}

export function withPresentation(
	node: EditorNode,
	lookup: EditorLookup,
): EditorNode {
	const defaults = defaultPresentation(node.data, lookup)
	const placed = node.data.connector ?? node.data.trigger
	return {
		...node,
		data: {
			...node.data,
			label: placed?.name || defaults.label,
			subtitle: placed?.description ?? defaults.subtitle,
			disabled: node.data.connector?.disabled ?? false,
			family: defaults.family,
			typeLabel: defaults.typeLabel,
		},
	}
}

export function withDetails(
	node: EditorNode,
	patch: NodeDetailsPatch,
	lookup: EditorLookup,
): EditorNode {
	const { connector, trigger } = node.data
	return withPresentation(
		{
			...node,
			data: {
				...node.data,
				...(connector ? { connector: { ...connector, ...patch } } : {}),
				...(trigger ? { trigger: { ...trigger, ...patch } } : {}),
			},
		},
		lookup,
	)
}

export function withRenamedReferences(
	nodes: readonly EditorNode[],
	renamedId: string,
	oldLabel: string,
	newLabel: string,
): EditorNode[] {
	return nodes.map((node) => {
		const connector = node.data.connector
		if (!connector || node.id === renamedId) return node
		const config = renameReferencesInValue(
			connector.config,
			oldLabel,
			newLabel,
		) as Record<string, unknown>
		return config === connector.config
			? node
			: { ...node, data: { ...node.data, connector: { ...connector, config } } }
	})
}

export function withDisabled(
	node: EditorNode,
	disabled: boolean,
	lookup: EditorLookup,
): EditorNode {
	const { connector } = node.data
	if (!connector) return node
	return withPresentation(
		{
			...node,
			data: { ...node.data, connector: { ...connector, disabled } },
		},
		lookup,
	)
}

function branchesOf(
	connector: Schemas.PlacedConnectorDto,
	lookup: EditorLookup,
): Schemas.BranchDto[] {
	return lookup.descriptors.get(connector.kind)?.branches ?? []
}

export function connectorNode(
	connector: Schemas.PlacedConnectorDto,
	position: NodePosition,
	lookup: EditorLookup,
	errors: ConnectorValidationError[] = [],
	runStatus: string | null = null,
): EditorNode {
	return withPresentation(
		{
			id: connector.id,
			type: 'automation',
			position,
			data: {
				kind: 'connector',
				label: '',
				subtitle: '',
				disabled: false,
				typeLabel: '',
				branches: branchesOf(connector, lookup),
				errors,
				runStatus,
				connector,
			},
		},
		lookup,
	)
}

export function triggerNode(
	trigger: Schemas.PlacedTriggerDto,
	position: NodePosition,
	lookup: EditorLookup,
	errors: ConnectorValidationError[] = [],
): EditorNode {
	return withPresentation(
		{
			id: trigger.id,
			type: 'automation',
			position,
			data: {
				kind: 'trigger',
				label: '',
				subtitle: '',
				disabled: false,
				typeLabel: '',
				branches: [],
				errors,
				runStatus: null,
				trigger,
			},
		},
		lookup,
	)
}

const TRIGGER_X_OFFSET = 220
const TRIGGER_Y_SPACING = 140

export function toEditorNodes(
	graph: Schemas.GraphDto,
	layout: Map<string, NodePosition>,
	lookup: EditorLookup,
	errors: Map<string, ConnectorValidationError[]>,
	runStatuses: Map<string, string>,
): EditorNode[] {
	const triggers = graph.triggers.map((trigger, index) =>
		triggerNode(
			trigger,
			layout.get(trigger.id) ?? {
				x: -TRIGGER_X_OFFSET,
				y: index * TRIGGER_Y_SPACING,
			},
			lookup,
			errors.get(trigger.id),
		),
	)
	const connectors = graph.connectors.map((connector) =>
		connectorNode(
			connector,
			layout.get(connector.id) ?? { x: 0, y: 0 },
			lookup,
			errors.get(connector.id),
			runStatuses.get(connector.id) ?? null,
		),
	)
	return [...triggers, ...connectors]
}

export function toEditorEdges(graph: Schemas.GraphDto): EditorEdge[] {
	return graph.edges.map((edge) => ({
		id: edgeId(edge),
		source: edge.from,
		target: edge.to,
		sourceHandle: edge.branch ?? undefined,
	}))
}

export function toGraph(
	nodes: readonly EditorNode[],
	edges: readonly EditorEdge[],
): Schemas.GraphDto {
	const connectors: Schemas.PlacedConnectorDto[] = []
	const triggers: Schemas.PlacedTriggerDto[] = []
	const read = referencedLabels(
		nodes.map((node) => node.data.connector?.config),
	)
	const named = <T extends { name?: string | null }>(
		placed: T,
		label: string,
	): T =>
		placed.name?.trim() || !read.has(label)
			? placed
			: { ...placed, name: label }
	for (const node of nodes) {
		if (node.data.connector) {
			connectors.push(named(node.data.connector, node.data.label))
		}
		if (node.data.trigger) {
			triggers.push(named(node.data.trigger, node.data.label))
		}
	}
	return {
		connectors,
		triggers,
		edges: edges.map((edge) => ({
			from: edge.source,
			to: edge.target,
			branch:
				(edge.sourceHandle as Schemas.BranchDto | null | undefined) ?? null,
		})),
	}
}

export function toLayout(
	nodes: readonly EditorNode[],
): Map<string, NodePosition> {
	return new Map(
		nodes.map((node) => [node.id, { x: node.position.x, y: node.position.y }]),
	)
}

export function outputHandles(data: EditorNodeData): (string | null)[] {
	return data.branches.length === 0 ? [null] : data.branches
}

export function isTriggerNode(node: EditorNode): boolean {
	return node.data.kind === 'trigger'
}

export interface OutputSource {
	nodeId: string
	outputId: string | null
}

export function findOpenOutput(
	node: EditorNode,
	edges: readonly EditorEdge[],
): OutputSource | null {
	const used = new Set(
		edges
			.filter((edge) => edge.source === node.id)
			.map((edge) => edge.sourceHandle ?? null),
	)
	const open = outputHandles(node.data).find((handle) => !used.has(handle))
	return open === undefined ? null : { nodeId: node.id, outputId: open }
}
