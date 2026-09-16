import {
	addEdge,
	applyEdgeChanges,
	applyNodeChanges,
	Background,
	type Connection,
	type Edge,
	type EdgeChange,
	type Node,
	type NodeChange,
	ReactFlow,
	ReactFlowProvider,
	useEdgesState,
	useNodesState,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	type AvailableConnectorData,
	buildAvailableDataTree,
	resolveTriggerExample,
	toEvaluateContext,
} from '#/pages/automation/lib/data-tree'
import type { NodePosition } from '#/pages/automation/lib/graph'
import {
	connectorsReferencing,
	nextConnectorId,
	nextNodePosition,
	removeConnector,
	rootConnectorIds,
	upstreamConnectorIds,
} from '#/pages/automation/lib/graph'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	ConnectorConfigPanel,
	type DataTreeSourceBundle,
} from '#/pages/automation/ui/connector-config-panel'
import {
	ConnectorNode,
	type ConnectorNodeData,
} from '#/pages/automation/ui/connector-node'
import { TriggerConfigPanel } from '#/pages/automation/ui/trigger-config-panel'
import {
	TriggerNode,
	type TriggerNodeData,
} from '#/pages/automation/ui/trigger-node'
import { WorkflowCanvasActionsContext } from '#/pages/automation/ui/workflow-canvas-context'

export const TRIGGER_NODE_ID = '__trigger__'
const TRIGGER_X_OFFSET = 220

const NODE_TYPES = {
	connector: ConnectorNode,
	trigger: TriggerNode,
}

type CreatedCredential = Schemas.CredentialResponse & { secret: unknown }

export interface LastRunData {
	triggerPayload: unknown
	connectorOutputs: Record<string, unknown>
}

export interface WorkflowCanvasProps {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors: Map<string, ConnectorValidationError[]>
	events: Schemas.EventDescriptorResponse[]
	triggerEventNames: string[]
	onSaveTrigger: (eventNames: string[]) => void
	isSavingTrigger: boolean
	triggerSaveError: string | null
	lastRun: LastRunData | null
	onEvaluateExpression: (
		template: unknown,
		context: Schemas.EvaluateContextBody,
	) => Promise<unknown>
	credentials?: Schemas.CredentialResponse[]
	authSchemes?: Schemas.AuthSchemeResponse[]
	onCreateCredential?: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
	isDirty: boolean
	isSaving: boolean
	onChange: (graph: Schemas.GraphDto, layout: Map<string, NodePosition>) => void
	onSave: () => void
}

function rejectCreateCredential(): Promise<CreatedCredential> {
	return Promise.reject(
		new Error('onCreateCredential was not wired for this canvas'),
	)
}

function triggerEdgeId(rootId: string): string {
	return `${TRIGGER_NODE_ID}->${rootId}`
}

function isTriggerEdgeId(id: string): boolean {
	return id.startsWith(`${TRIGGER_NODE_ID}->`)
}

function realEdgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

function triggerPosition(
	graph: Schemas.GraphDto,
	positions: Map<string, NodePosition>,
): NodePosition {
	const roots = rootConnectorIds(graph)
	if (roots.length === 0) return { x: -TRIGGER_X_OFFSET, y: 0 }

	const sumY = roots.reduce((sum, id) => sum + (positions.get(id)?.y ?? 0), 0)
	return { x: -TRIGGER_X_OFFSET, y: sumY / roots.length }
}

function buildInitialNodes(
	graph: Schemas.GraphDto,
	layout: Map<string, NodePosition>,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
	connectorErrors: Map<string, ConnectorValidationError[]>,
	triggerEventNames: string[],
): Node[] {
	const triggerNode: Node = {
		id: TRIGGER_NODE_ID,
		type: 'trigger',
		position: triggerPosition(graph, layout),
		data: { hasEvent: triggerEventNames.length > 0 },
		deletable: false,
		draggable: false,
	}

	const connectorNodes: Node[] = graph.connectors.map((connector) => {
		const descriptor = descriptors.get(connector.kind)
		const data: ConnectorNodeData = {
			label: descriptor?.label ?? connector.id,
			branches: descriptor?.branches ?? [],
			errors: connectorErrors.get(connector.id) ?? [],
			connector,
		}

		return {
			id: connector.id,
			type: 'connector',
			position: layout.get(connector.id) ?? { x: 0, y: 0 },
			data,
		}
	})

	return [triggerNode, ...connectorNodes]
}

function buildInitialEdges(graph: Schemas.GraphDto): Edge[] {
	const triggerEdges: Edge[] = rootConnectorIds(graph).map((rootId) => ({
		id: triggerEdgeId(rootId),
		source: TRIGGER_NODE_ID,
		target: rootId,
		deletable: false,
		reconnectable: false,
	}))

	const realEdges: Edge[] = graph.edges.map((edge) => ({
		id: realEdgeId(edge),
		source: edge.from,
		target: edge.to,
		sourceHandle: edge.branch ?? undefined,
	}))

	return [...triggerEdges, ...realEdges]
}

function buildGraph(nodes: Node[], edges: Edge[]): Schemas.GraphDto {
	const connectors = nodes
		.filter((node) => node.id !== TRIGGER_NODE_ID)
		.map((node) => (node.data as ConnectorNodeData).connector)

	const graphEdges: Schemas.EdgeDto[] = edges
		.filter((edge) => edge.source !== TRIGGER_NODE_ID)
		.map((edge) => ({
			from: edge.source,
			to: edge.target,
			branch: (edge.sourceHandle as Schemas.BranchDto | undefined) ?? null,
		}))

	return { connectors, edges: graphEdges }
}

function buildLayout(nodes: Node[]): Map<string, NodePosition> {
	const layout = new Map<string, NodePosition>()
	for (const node of nodes) {
		if (node.id === TRIGGER_NODE_ID) continue
		layout.set(node.id, { x: node.position.x, y: node.position.y })
	}
	return layout
}

export interface BranchConnection {
	source: string
	target: string
	sourceHandle?: string | null
	targetHandle?: string | null
}

export function isBranchDeclared(
	connection: BranchConnection,
	graph: Schemas.GraphDto,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
): boolean {
	if (connection.source === TRIGGER_NODE_ID) return false

	const source = graph.connectors.find(
		(connector) => connector.id === connection.source,
	)
	if (!source) return false

	const branches = descriptors.get(source.kind)?.branches ?? []
	if (branches.length === 0) return connection.sourceHandle == null

	return (
		connection.sourceHandle != null &&
		branches.includes(connection.sourceHandle as Schemas.BranchDto)
	)
}

export function WorkflowCanvas({
	graph,
	layout,
	descriptors,
	connectorErrors,
	events,
	triggerEventNames,
	onSaveTrigger,
	isSavingTrigger,
	triggerSaveError,
	lastRun,
	onEvaluateExpression,
	credentials = [],
	authSchemes = [],
	onCreateCredential = rejectCreateCredential,
	isDirty,
	isSaving,
	onChange,
	onSave,
}: WorkflowCanvasProps) {
	const [nodes, setNodes, onNodesChangeInternal] = useNodesState(
		buildInitialNodes(
			graph,
			layout,
			descriptors,
			connectorErrors,
			triggerEventNames,
		),
	)
	const [edges, setEdges] = useEdgesState(buildInitialEdges(graph))
	const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null)
	const [openConnectorId, setOpenConnectorId] = useState<string | null>(null)
	const [openTrigger, setOpenTrigger] = useState(false)

	const nodesRef = useRef(nodes)
	nodesRef.current = nodes
	const edgesRef = useRef(edges)
	edgesRef.current = edges

	const catalogue = useMemo(() => [...descriptors.values()], [descriptors])

	useEffect(() => {
		setNodes((current) =>
			current.map((node) => {
				if (node.id === TRIGGER_NODE_ID) return node
				const data = node.data as ConnectorNodeData
				const errors = connectorErrors.get(node.id) ?? []
				if (data.errors === errors) return node
				return { ...node, data: { ...data, errors } }
			}),
		)
	}, [connectorErrors, setNodes])

	useEffect(() => {
		const hasEvent = triggerEventNames.length > 0
		setNodes((current) =>
			current.map((node) => {
				if (node.id !== TRIGGER_NODE_ID) return node
				const data = node.data as TriggerNodeData
				if (data.hasEvent === hasEvent) return node
				return { ...node, data: { ...data, hasEvent } }
			}),
		)
	}, [triggerEventNames, setNodes])

	const handleNodesChange = useCallback(
		(changes: NodeChange[]) => {
			const applicable = changes.filter((change) => change.type !== 'remove')
			nodesRef.current = applyNodeChanges(applicable, nodesRef.current)
			onNodesChangeInternal(applicable)
		},
		[onNodesChangeInternal],
	)

	const handleEdgesChange = useCallback(
		(changes: EdgeChange[]) => {
			const applicable = changes.filter(
				(change) => !(change.type === 'remove' && isTriggerEdgeId(change.id)),
			)
			const nextEdges = applyEdgeChanges(applicable, edgesRef.current)
			edgesRef.current = nextEdges
			setEdges(nextEdges)
			if (applicable.some((change) => change.type === 'remove')) {
				onChange(
					buildGraph(nodesRef.current, nextEdges),
					buildLayout(nodesRef.current),
				)
			}
		},
		[onChange, setEdges],
	)

	const handleConnect = useCallback(
		(connection: Connection) => {
			const nextEdges = addEdge(connection, edgesRef.current)
			edgesRef.current = nextEdges
			setEdges(nextEdges)
			onChange(
				buildGraph(nodesRef.current, nextEdges),
				buildLayout(nodesRef.current),
			)
		},
		[onChange, setEdges],
	)

	const handleNodeDragStop = useCallback(() => {
		onChange(
			buildGraph(nodesRef.current, edgesRef.current),
			buildLayout(nodesRef.current),
		)
	}, [onChange])

	const validateConnection = useCallback(
		(connection: Connection | Edge) =>
			isBranchDeclared(
				connection,
				buildGraph(nodesRef.current, edgesRef.current),
				descriptors,
			),
		[descriptors],
	)

	const handleAddNode = useCallback(
		(
			sourceId: string,
			branch: Schemas.BranchDto | null,
			descriptor: Schemas.ConnectorDescriptorResponse,
		) => {
			const currentGraph = buildGraph(nodesRef.current, edgesRef.current)
			const newId = nextConnectorId(currentGraph)
			const newConnector: Schemas.PlacedConnectorDto = {
				id: newId,
				kind: descriptor.kind,
				version: descriptor.version,
				config: {},
			}
			const nextGraph: Schemas.GraphDto = {
				connectors: [...currentGraph.connectors, newConnector],
				edges:
					sourceId === TRIGGER_NODE_ID
						? currentGraph.edges
						: [
								...currentGraph.edges,
								{ from: sourceId, to: newId, branch: branch ?? null },
							],
			}

			const sourcePosition =
				nodesRef.current.find((node) => node.id === sourceId)?.position ??
				({ x: 0, y: 0 } as NodePosition)
			const siblingIndex = edgesRef.current.filter(
				(edge) => edge.source === sourceId,
			).length
			const position = nextNodePosition(sourcePosition, siblingIndex)

			const data: ConnectorNodeData = {
				label: descriptor.label,
				branches: descriptor.branches,
				errors: [],
				connector: newConnector,
			}
			const nextNodes = [
				...nodesRef.current,
				{ id: newId, type: 'connector', position, data },
			]
			const nextEdges = buildInitialEdges(nextGraph)

			nodesRef.current = nextNodes
			edgesRef.current = nextEdges
			setNodes(nextNodes)
			setEdges(nextEdges)
			onChange(nextGraph, buildLayout(nextNodes))
		},
		[onChange, setNodes, setEdges],
	)

	const handleRequestDelete = useCallback((connectorId: string) => {
		setPendingDeleteId(connectorId)
	}, [])

	const handleConfirmDelete = useCallback(() => {
		if (!pendingDeleteId) return

		const currentGraph = buildGraph(nodesRef.current, edgesRef.current)
		const nextGraph = removeConnector(currentGraph, pendingDeleteId)
		const nextNodes = nodesRef.current.filter(
			(node) => node.id !== pendingDeleteId,
		)
		const nextEdges = buildInitialEdges(nextGraph)

		nodesRef.current = nextNodes
		edgesRef.current = nextEdges
		setNodes(nextNodes)
		setEdges(nextEdges)
		onChange(nextGraph, buildLayout(nextNodes))
		setOpenConnectorId((current) =>
			current === pendingDeleteId ? null : current,
		)
		setPendingDeleteId(null)
	}, [pendingDeleteId, onChange, setNodes, setEdges])

	const handleConnectorConfigChange = useCallback(
		(
			connectorId: string,
			patch: Partial<
				Pick<Schemas.PlacedConnectorDto, 'config' | 'credential_id'>
			>,
		) => {
			const nextNodes = nodesRef.current.map((node) => {
				if (node.id !== connectorId) return node
				const data = node.data as ConnectorNodeData
				return {
					...node,
					data: { ...data, connector: { ...data.connector, ...patch } },
				}
			})
			nodesRef.current = nextNodes
			setNodes(nextNodes)
			onChange(buildGraph(nextNodes, edgesRef.current), buildLayout(nextNodes))
		},
		[onChange, setNodes],
	)

	const handleNodeClick = useCallback((_event: unknown, node: Node) => {
		if (node.type === 'trigger') {
			setOpenConnectorId(null)
			setOpenTrigger(true)
			return
		}
		if (node.type !== 'connector') return
		setOpenTrigger(false)
		setOpenConnectorId(node.id)
	}, [])

	const pendingDeleteNode = pendingDeleteId
		? nodes.find((node) => node.id === pendingDeleteId)
		: undefined
	const pendingDeleteLabel = pendingDeleteNode
		? (pendingDeleteNode.data as ConnectorNodeData).label
		: pendingDeleteId
	const pendingDeleteReferences = pendingDeleteId
		? connectorsReferencing(buildGraph(nodes, edges), pendingDeleteId).map(
				(id) => {
					const referencing = nodes.find((node) => node.id === id)
					return referencing
						? (referencing.data as ConnectorNodeData).label
						: id
				},
			)
		: []

	const actions = useMemo(
		() => ({
			catalogue,
			onAddNode: handleAddNode,
			onRequestDelete: handleRequestDelete,
		}),
		[catalogue, handleAddNode, handleRequestDelete],
	)

	const openNode = openConnectorId
		? nodes.find((node) => node.id === openConnectorId)
		: undefined
	const openConnectorData = openNode?.data as ConnectorNodeData | undefined
	const openDescriptor = openConnectorData
		? descriptors.get(openConnectorData.connector.kind)
		: undefined

	function connectorLabel(id: string): string {
		const data = nodes.find((node) => node.id === id)?.data as
			| ConnectorNodeData
			| undefined
		return data?.label ?? id
	}

	function connectorOutputExample(id: string): unknown {
		const data = nodes.find((node) => node.id === id)?.data as
			| ConnectorNodeData
			| undefined
		if (!data) return null
		return descriptors.get(data.connector.kind)?.output_example ?? null
	}

	const upstreamIds = openConnectorId
		? upstreamConnectorIds(buildGraph(nodes, edges), openConnectorId)
		: []

	const triggerExample = resolveTriggerExample(events, triggerEventNames)
	const exampleConnectors: AvailableConnectorData[] = upstreamIds.map((id) => ({
		id,
		label: connectorLabel(id),
		output: connectorOutputExample(id),
	}))
	const exampleData: DataTreeSourceBundle = {
		tree: buildAvailableDataTree({
			trigger: triggerExample,
			connectors: exampleConnectors,
		}),
		context: toEvaluateContext({
			trigger: triggerExample,
			connectors: exampleConnectors,
		}),
	}

	const lastRunConnectors: AvailableConnectorData[] = lastRun
		? upstreamIds.map((id) => ({
				id,
				label: connectorLabel(id),
				output: lastRun.connectorOutputs[id] ?? null,
			}))
		: []
	const lastRunData: DataTreeSourceBundle | null = lastRun
		? {
				tree: buildAvailableDataTree({
					trigger: lastRun.triggerPayload,
					connectors: lastRunConnectors,
				}),
				context: toEvaluateContext({
					trigger: lastRun.triggerPayload,
					connectors: lastRunConnectors,
				}),
			}
		: null

	return (
		<WorkflowCanvasActionsContext.Provider value={actions}>
			<div className="flex min-h-0 flex-1 flex-col">
				<div className="flex items-center justify-between border-b px-4 py-2">
					<span className="text-xs text-muted-foreground">
						{isDirty ? 'Modifications non enregistrées' : 'Enregistré'}
					</span>
					<Button size="sm" onClick={onSave} disabled={isSaving}>
						{isSaving ? 'Enregistrement…' : 'Enregistrer'}
					</Button>
				</div>
				<div className="flex min-h-0 flex-1">
					<div className="min-h-0 min-w-0 flex-1">
						<ReactFlowProvider>
							<ReactFlow
								nodes={nodes}
								edges={edges}
								nodeTypes={NODE_TYPES}
								onNodesChange={handleNodesChange}
								onEdgesChange={handleEdgesChange}
								onConnect={handleConnect}
								onNodeDragStop={handleNodeDragStop}
								onNodeClick={handleNodeClick}
								onPaneClick={() => {
									setOpenConnectorId(null)
									setOpenTrigger(false)
								}}
								isValidConnection={validateConnection}
								autoPanOnNodeDrag={false}
								fitView={false}
							>
								<Background />
							</ReactFlow>
						</ReactFlowProvider>
					</div>
					{openConnectorId && openConnectorData && openDescriptor ? (
						<ConnectorConfigPanel
							connectorId={openConnectorId}
							label={openConnectorData.label}
							descriptor={openDescriptor}
							config={openConnectorData.connector.config}
							credentialId={openConnectorData.connector.credential_id ?? null}
							credentials={credentials}
							authSchemes={authSchemes}
							errors={openConnectorData.errors}
							onClose={() => setOpenConnectorId(null)}
							onConfigChange={(config) =>
								handleConnectorConfigChange(openConnectorId, { config })
							}
							onCredentialChange={(credentialId) =>
								handleConnectorConfigChange(openConnectorId, {
									credential_id: credentialId,
								})
							}
							onCreateCredential={onCreateCredential}
							exampleData={exampleData}
							lastRunData={lastRunData}
							onEvaluateExpression={onEvaluateExpression}
						/>
					) : null}
					{openTrigger ? (
						<TriggerConfigPanel
							events={events}
							selectedEventNames={triggerEventNames}
							isSaving={isSavingTrigger}
							saveError={triggerSaveError}
							onClose={() => setOpenTrigger(false)}
							onSave={onSaveTrigger}
						/>
					) : null}
				</div>
				<AlertDialog
					open={pendingDeleteId !== null}
					onOpenChange={(open) => {
						if (!open) setPendingDeleteId(null)
					}}
				>
					<AlertDialogContent>
						<AlertDialogHeader>
							<AlertDialogTitle>
								Supprimer {pendingDeleteLabel} ?
							</AlertDialogTitle>
							<AlertDialogDescription>
								{pendingDeleteReferences.length > 0
									? `Les connecteurs suivants font référence à celui-ci dans leurs expressions : ${pendingDeleteReferences.join(', ')}. Ces expressions cesseront de fonctionner après la suppression.`
									: 'Ce connecteur et ses connexions seront retirés du graphe.'}
							</AlertDialogDescription>
						</AlertDialogHeader>
						<AlertDialogFooter>
							<AlertDialogCancel>Annuler</AlertDialogCancel>
							<AlertDialogAction onClick={handleConfirmDelete}>
								Supprimer
							</AlertDialogAction>
						</AlertDialogFooter>
					</AlertDialogContent>
				</AlertDialog>
			</div>
		</WorkflowCanvasActionsContext.Provider>
	)
}
