import type { Edge, Node } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import {
	connectorCopy,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import type { NodePosition } from '#/pages/automation/lib/graph'
import {
	isManualTrigger,
	triggerPresentation,
} from '#/pages/automation/lib/trigger-events'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'

export interface NodeOperationRef {
	collectionId: string
	operationId: string
}

export interface NodeMeta {
	label?: string
	subtitle?: string
	disabled?: boolean
	operation?: NodeOperationRef
}

export type NodeMetaMap = Record<string, NodeMeta>

export interface EditorLookup {
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	events: readonly Schemas.EventDescriptorResponse[]
}

export interface EditorNodeData extends Record<string, unknown> {
	kind: 'connector' | 'trigger'
	label: string
	subtitle: string
	disabled: boolean
	meta: NodeMeta
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
			typeLabel: triggerCopy(
				isManualTrigger(data.trigger) ? 'manual' : 'events',
			).label,
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
	const meta = node.data.meta
	return {
		...node,
		data: {
			...node.data,
			label: meta.label ?? defaults.label,
			subtitle: meta.subtitle ?? defaults.subtitle,
			disabled: meta.disabled ?? false,
			family: defaults.family,
			typeLabel: defaults.typeLabel,
		},
	}
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
	meta: NodeMeta = {},
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
				meta,
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
	meta: NodeMeta = {},
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
				meta,
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
	meta: NodeMetaMap,
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
			meta[trigger.id],
			errors.get(trigger.id),
		),
	)
	const connectors = graph.connectors.map((connector) =>
		connectorNode(
			connector,
			layout.get(connector.id) ?? { x: 0, y: 0 },
			lookup,
			meta[connector.id],
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
	for (const node of nodes) {
		if (node.data.connector) connectors.push(node.data.connector)
		if (node.data.trigger) triggers.push(node.data.trigger)
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

export function toMeta(nodes: readonly EditorNode[]): NodeMetaMap {
	const meta: NodeMetaMap = {}
	for (const node of nodes) {
		const { label, subtitle, disabled, operation } = node.data.meta
		if (
			label !== undefined ||
			subtitle !== undefined ||
			disabled ||
			operation !== undefined
		) {
			meta[node.id] = node.data.meta
		}
	}
	return meta
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
