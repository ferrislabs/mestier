import {
	addEdge,
	applyEdgeChanges,
	applyNodeChanges,
	type Connection,
	type EdgeChange,
	type NodeChange,
} from '@xyflow/react'
import { useCallback, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { withFieldDefaults } from '#/pages/automation/lib/connector-field-copy'
import {
	connectorNode,
	type EditorEdge,
	type EditorLookup,
	type EditorNode,
	edgeIdOf,
	type NodeDetailsPatch,
	type OutputSource,
	outputHandles,
	toEditorEdges,
	toEditorNodes,
	toGraph,
	toLayout,
	triggerNode,
	withDetails,
	withDisabled,
	withPresentation,
} from '#/pages/automation/lib/editor-nodes'
import {
	findFreePosition,
	type NodePosition,
	nextConnectorId,
	nextTriggerId,
	placementAfter,
	type Reconnection,
	snapToGrid,
} from '#/pages/automation/lib/graph'
import {
	emptyHistory,
	type History,
	type LastRecord,
	pushHistory,
	redoHistory,
	shouldCoalesce,
	undoHistory,
} from '#/pages/automation/lib/history'
import { uniqueLabel } from '#/pages/automation/lib/references'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'

const STRUCTURAL_CHANGES: ReadonlySet<string> = new Set([
	'position',
	'remove',
	'add',
	'replace',
])

function sameErrors(
	a: readonly ConnectorValidationError[],
	b: readonly ConnectorValidationError[],
): boolean {
	return a.length === b.length && a.every((error, index) => error === b[index])
}

export interface ConnectorPreset {
	config?: Record<string, unknown>
	operation?: Schemas.OperationRefDto
	name?: string
	description?: string
}

export interface GraphChange {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
}

interface Snapshot {
	nodes: EditorNode[]
	edges: EditorEdge[]
}

export interface WorkflowGraphState {
	nodes: EditorNode[]
	edges: EditorEdge[]
	onNodesChange: (changes: NodeChange<EditorNode>[]) => void
	onEdgesChange: (changes: EdgeChange<EditorEdge>[]) => void
	onConnect: (connection: Connection) => void
	addConnector: (
		descriptor: Schemas.ConnectorDescriptorResponse,
		source: OutputSource | null,
		fallbackPosition: NodePosition,
		preset?: ConnectorPreset,
	) => string
	addTrigger: (
		kind: Schemas.TriggerKindDto,
		fallbackPosition: NodePosition,
	) => string
	updateConnector: (
		nodeId: string,
		patch: Partial<
			Pick<Schemas.PlacedConnectorDto, 'config' | 'credential_id'>
		>,
	) => void
	updateTriggerKind: (nodeId: string, kind: Schemas.TriggerKindDto) => void
	updateDetails: (nodeId: string, patch: NodeDetailsPatch) => void
	selectNode: (nodeId: string) => void
	setDisabled: (nodeIds: readonly string[], disabled: boolean) => void
	applyPositions: (positions: ReadonlyMap<string, NodePosition>) => void
	reconnect: (reconnections: readonly Reconnection[]) => void
	insertCopy: (
		source: EditorNode,
		position: NodePosition,
		label: string,
	) => void
	syncExternal: (
		errors: Map<string, ConnectorValidationError[]>,
		runStatuses: Map<string, string>,
	) => void
	record: (key?: string) => void
	undo: () => void
	redo: () => void
	canUndo: boolean
	canRedo: boolean
}

export interface UseWorkflowGraphOptions {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	lookup: EditorLookup
	errors: Map<string, ConnectorValidationError[]>
	runStatuses: Map<string, string>
	onChange: (change: GraphChange) => void
}

export function useWorkflowGraph({
	graph,
	layout,
	lookup,
	errors,
	runStatuses,
	onChange,
}: UseWorkflowGraphOptions): WorkflowGraphState {
	const [initial] = useState<Snapshot>(() => ({
		nodes: toEditorNodes(graph, layout, lookup, errors, runStatuses),
		edges: toEditorEdges(graph),
	}))
	const [state, setState] = useState<Snapshot>(initial)
	const stateRef = useRef<Snapshot>(initial)
	const lookupRef = useRef(lookup)
	lookupRef.current = lookup
	const onChangeRef = useRef(onChange)
	onChangeRef.current = onChange
	const historyRef = useRef<History<Snapshot>>(emptyHistory())
	const lastRecord = useRef<LastRecord>(null)
	const [, setHistoryVersion] = useState(0)

	const commit = useCallback((next: Snapshot, notify: boolean) => {
		stateRef.current = next
		setState(next)
		if (notify) {
			onChangeRef.current({
				graph: toGraph(next.nodes, next.edges),
				layout: toLayout(next.nodes),
			})
		}
	}, [])

	const record = useCallback((key?: string) => {
		const now = Date.now()
		if (shouldCoalesce(lastRecord.current, key, now)) {
			lastRecord.current = key ? { key, at: now } : null
			return
		}
		lastRecord.current = key ? { key, at: now } : null
		historyRef.current = pushHistory(historyRef.current, stateRef.current)
		setHistoryVersion((version) => version + 1)
	}, [])

	const restore = useCallback(
		(direction: 'undo' | 'redo') => {
			const step =
				direction === 'undo'
					? undoHistory(historyRef.current, stateRef.current)
					: redoHistory(historyRef.current, stateRef.current)
			if (!step) return
			historyRef.current = step.history
			lastRecord.current = null
			setHistoryVersion((version) => version + 1)
			commit(step.snapshot, true)
		},
		[commit],
	)

	const undo = useCallback(() => restore('undo'), [restore])
	const redo = useCallback(() => restore('redo'), [restore])

	const onNodesChange = useCallback(
		(changes: NodeChange<EditorNode>[]) => {
			const { nodes, edges } = stateRef.current
			commit(
				{ nodes: applyNodeChanges(changes, nodes), edges },
				changes.some((change) => STRUCTURAL_CHANGES.has(change.type)),
			)
		},
		[commit],
	)

	const onEdgesChange = useCallback(
		(changes: EdgeChange<EditorEdge>[]) => {
			const { nodes, edges } = stateRef.current
			commit(
				{ nodes, edges: applyEdgeChanges(changes, edges) },
				changes.some((change) => STRUCTURAL_CHANGES.has(change.type)),
			)
		},
		[commit],
	)

	const onConnect = useCallback(
		(connection: Connection) => {
			record()
			const { nodes, edges } = stateRef.current
			commit(
				{
					nodes,
					edges: addEdge(
						{
							...connection,
							sourceHandle: connection.sourceHandle ?? undefined,
							id: edgeIdOf(
								connection.source,
								connection.target,
								connection.sourceHandle,
							),
						},
						edges,
					),
				},
				true,
			)
		},
		[commit, record],
	)

	const selectOnly = useCallback(
		(nodes: EditorNode[], selectedId: string): EditorNode[] =>
			nodes.map((node) => ({ ...node, selected: node.id === selectedId })),
		[],
	)

	const addConnector = useCallback(
		(
			descriptor: Schemas.ConnectorDescriptorResponse,
			source: OutputSource | null,
			fallbackPosition: NodePosition,
			preset?: ConnectorPreset,
		) => {
			record()
			const { nodes, edges } = stateRef.current
			const id = nextConnectorId(toGraph(nodes, edges))
			const sourceNode = source
				? nodes.find((node) => node.id === source.nodeId)
				: undefined
			const origin = sourceNode
				? (() => {
						const handles = outputHandles(sourceNode.data)
						return placementAfter(
							sourceNode.position,
							Math.max(0, handles.indexOf(source?.outputId ?? null)),
							handles.length,
						)
					})()
				: fallbackPosition
			const connector: Schemas.PlacedConnectorDto = {
				id,
				kind: descriptor.kind,
				version: descriptor.version,
				config: preset?.config ?? withFieldDefaults(descriptor.kind, {}) ?? {},
				...(preset?.name === undefined ? {} : { name: preset.name }),
				...(preset?.description === undefined
					? {}
					: { description: preset.description }),
				...(preset?.operation === undefined
					? {}
					: { operation: preset.operation }),
			}
			const created = connectorNode(
				connector,
				findFreePosition(snapToGrid(origin), nodes),
				lookupRef.current,
			)
			const label = uniqueLabel(
				created.data.label,
				nodes.map((node) => node.data.label),
			)
			const named =
				label === created.data.label
					? created
					: withDetails(created, { name: label }, lookupRef.current)
			commit(
				{
					nodes: [...selectOnly(nodes, id), { ...named, selected: true }],
					edges:
						source && sourceNode
							? [
									...edges,
									{
										id: edgeIdOf(source.nodeId, id, source.outputId),
										source: source.nodeId,
										sourceHandle: source.outputId ?? undefined,
										target: id,
									},
								]
							: edges,
				},
				true,
			)
			return id
		},
		[commit, record, selectOnly],
	)

	const addTrigger = useCallback(
		(kind: Schemas.TriggerKindDto, fallbackPosition: NodePosition) => {
			record()
			const { nodes, edges } = stateRef.current
			const id = nextTriggerId(toGraph(nodes, edges))
			const created = triggerNode(
				{ id, kind },
				findFreePosition(snapToGrid(fallbackPosition), nodes),
				lookupRef.current,
			)
			const label = uniqueLabel(
				created.data.label,
				nodes.map((node) => node.data.label),
			)
			const named =
				label === created.data.label
					? created
					: withDetails(created, { name: label }, lookupRef.current)
			commit(
				{
					nodes: [...selectOnly(nodes, id), { ...named, selected: true }],
					edges,
				},
				true,
			)
			return id
		},
		[commit, record, selectOnly],
	)

	const mapNode = useCallback(
		(nodeId: string, change: (node: EditorNode) => EditorNode) => {
			const { nodes, edges } = stateRef.current
			commit(
				{
					nodes: nodes.map((node) =>
						node.id === nodeId ? change(node) : node,
					),
					edges,
				},
				true,
			)
		},
		[commit],
	)

	const updateConnector = useCallback(
		(
			nodeId: string,
			patch: Partial<
				Pick<Schemas.PlacedConnectorDto, 'config' | 'credential_id'>
			>,
		) => {
			record(`data:${nodeId}`)
			mapNode(nodeId, (node) =>
				node.data.connector
					? {
							...node,
							data: {
								...node.data,
								connector: { ...node.data.connector, ...patch },
							},
						}
					: node,
			)
		},
		[mapNode, record],
	)

	const updateTriggerKind = useCallback(
		(nodeId: string, kind: Schemas.TriggerKindDto) => {
			record(`data:${nodeId}`)
			mapNode(nodeId, (node) =>
				node.data.trigger
					? withPresentation(
							{
								...node,
								data: { ...node.data, trigger: { ...node.data.trigger, kind } },
							},
							lookupRef.current,
						)
					: node,
			)
		},
		[mapNode, record],
	)

	const updateDetails = useCallback(
		(nodeId: string, patch: NodeDetailsPatch) => {
			record(`meta:${nodeId}`)
			mapNode(nodeId, (node) => withDetails(node, patch, lookupRef.current))
		},
		[mapNode, record],
	)

	const setDisabled = useCallback(
		(nodeIds: readonly string[], disabled: boolean) => {
			const targets = new Set(nodeIds)
			const { nodes, edges } = stateRef.current
			if (!nodes.some((node) => targets.has(node.id) && node.data.connector))
				return
			record()
			commit(
				{
					nodes: nodes.map((node) =>
						targets.has(node.id)
							? withDisabled(node, disabled, lookupRef.current)
							: node,
					),
					edges,
				},
				true,
			)
		},
		[commit, record],
	)

	const selectNode = useCallback(
		(nodeId: string) => {
			const { nodes, edges } = stateRef.current
			commit({ nodes: selectOnly(nodes, nodeId), edges }, false)
		},
		[commit, selectOnly],
	)

	const reconnect = useCallback(
		(reconnections: readonly Reconnection[]) => {
			if (reconnections.length === 0) return
			const { nodes, edges } = stateRef.current
			commit(
				{
					nodes,
					edges: [
						...edges,
						...reconnections.map((reconnection) => ({
							id: edgeIdOf(
								reconnection.source,
								reconnection.target,
								reconnection.sourceHandle,
							),
							source: reconnection.source,
							sourceHandle: reconnection.sourceHandle ?? undefined,
							target: reconnection.target,
						})),
					],
				},
				true,
			)
		},
		[commit],
	)

	const applyPositions = useCallback(
		(positions: ReadonlyMap<string, NodePosition>) => {
			record()
			const { nodes, edges } = stateRef.current
			commit(
				{
					nodes: nodes.map((node) => {
						const position = positions.get(node.id)
						return position ? { ...node, position } : node
					}),
					edges,
				},
				true,
			)
		},
		[commit, record],
	)

	const insertCopy = useCallback(
		(source: EditorNode, position: NodePosition, label: string) => {
			record()
			const { nodes, edges } = stateRef.current
			const current = toGraph(nodes, edges)
			const slot = findFreePosition(snapToGrid(position), nodes)
			const taken = nodes.map((node) => node.data.label)
			const name = uniqueLabel(label, taken)
			const copy = source.data.connector
				? connectorNode(
						{
							...source.data.connector,
							id: nextConnectorId(current),
							config: structuredClone(source.data.connector.config),
							name,
						},
						slot,
						lookupRef.current,
					)
				: source.data.trigger
					? triggerNode(
							{
								...source.data.trigger,
								id: nextTriggerId(current),
								kind: structuredClone(source.data.trigger.kind),
								name,
							},
							slot,
							lookupRef.current,
						)
					: null
			if (!copy) return
			commit(
				{
					nodes: [...selectOnly(nodes, copy.id), { ...copy, selected: true }],
					edges,
				},
				true,
			)
		},
		[commit, record, selectOnly],
	)

	const syncExternal = useCallback(
		(
			nextErrors: Map<string, ConnectorValidationError[]>,
			nextStatuses: Map<string, string>,
		) => {
			const { nodes, edges } = stateRef.current
			const unchanged = nodes.every(
				(node) =>
					node.data.runStatus === (nextStatuses.get(node.id) ?? null) &&
					sameErrors(node.data.errors, nextErrors.get(node.id) ?? []),
			)
			if (unchanged) return
			commit(
				{
					nodes: nodes.map((node) => ({
						...node,
						data: {
							...node.data,
							errors: nextErrors.get(node.id) ?? [],
							runStatus: nextStatuses.get(node.id) ?? null,
						},
					})),
					edges,
				},
				false,
			)
		},
		[commit],
	)

	return {
		nodes: state.nodes,
		edges: state.edges,
		onNodesChange,
		onEdgesChange,
		onConnect,
		addConnector,
		addTrigger,
		updateConnector,
		updateTriggerKind,
		updateDetails,
		selectNode,
		setDisabled,
		applyPositions,
		reconnect,
		insertCopy,
		syncExternal,
		record,
		undo,
		redo,
		canUndo: historyRef.current.past.length > 0,
		canRedo: historyRef.current.future.length > 0,
	}
}
