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
	type ReactFlowInstance,
	ReactFlowProvider,
	useEdgesState,
	useNodesState,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import {
	type MouseEvent as ReactMouseEvent,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react'
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
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuSub,
	ContextMenuSubContent,
	ContextMenuSubTrigger,
	ContextMenuTrigger,
} from '#/components/ui/context-menu'
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
	nextTriggerId,
	removeConnector,
	removeTrigger,
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
import { ConnectorSearch } from '#/pages/automation/ui/connector-search'
import { DeletableEdge } from '#/pages/automation/ui/deletable-edge'
import { TriggerConfigPanel } from '#/pages/automation/ui/trigger-config-panel'
import {
	TriggerNode,
	type TriggerNodeData,
	triggerEventNames,
} from '#/pages/automation/ui/trigger-node'
import { WorkflowCanvasActionsContext } from '#/pages/automation/ui/workflow-canvas-context'

const TRIGGER_X_OFFSET = 220
const TRIGGER_Y_SPACING = 140
const FIT_VIEW_OPTIONS = { padding: 0.25, maxZoom: 1 }
const NODE_CENTER_X = 90
const NODE_CENTER_Y = 30
const CENTER_ON_NODE = { duration: 250, zoom: 1 }

const NODE_TYPES = {
	connector: ConnectorNode,
	trigger: TriggerNode,
}

const EDGE_TYPES = {
	deletable: DeletableEdge,
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
	lastRun: LastRunData | null
	runStatuses: Map<string, string>
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

function realEdgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

function buildInitialNodes(
	graph: Schemas.GraphDto,
	layout: Map<string, NodePosition>,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
	connectorErrors: Map<string, ConnectorValidationError[]>,
	runStatuses: Map<string, string>,
): Node[] {
	const triggerNodes: Node[] = graph.triggers.map((trigger, index) => ({
		id: trigger.id,
		type: 'trigger',
		position: layout.get(trigger.id) ?? {
			x: -TRIGGER_X_OFFSET,
			y: index * TRIGGER_Y_SPACING,
		},
		data: {
			trigger,
			errors: connectorErrors.get(trigger.id) ?? [],
		} satisfies TriggerNodeData,
	}))

	const connectorNodes: Node[] = graph.connectors.map((connector) => {
		const descriptor = descriptors.get(connector.kind)
		const data: ConnectorNodeData = {
			label: descriptor?.label ?? connector.id,
			branches: descriptor?.branches ?? [],
			errors: connectorErrors.get(connector.id) ?? [],
			runStatus: runStatuses.get(connector.id) ?? null,
			connector,
		}

		return {
			id: connector.id,
			type: 'connector',
			position: layout.get(connector.id) ?? { x: 0, y: 0 },
			data,
		}
	})

	return [...triggerNodes, ...connectorNodes]
}

function buildInitialEdges(graph: Schemas.GraphDto): Edge[] {
	return graph.edges.map((edge) => ({
		id: realEdgeId(edge),
		type: 'deletable',
		source: edge.from,
		target: edge.to,
		sourceHandle: edge.branch ?? undefined,
	}))
}

function buildGraph(nodes: Node[], edges: Edge[]): Schemas.GraphDto {
	const connectors = nodes
		.filter((node) => node.type === 'connector')
		.map((node) => (node.data as ConnectorNodeData).connector)

	const triggers = nodes
		.filter((node) => node.type === 'trigger')
		.map((node) => (node.data as TriggerNodeData).trigger)

	const graphEdges: Schemas.EdgeDto[] = edges.map((edge) => ({
		from: edge.source,
		to: edge.target,
		branch: (edge.sourceHandle as Schemas.BranchDto | undefined) ?? null,
	}))

	return { connectors, edges: graphEdges, triggers }
}

function buildLayout(nodes: Node[]): Map<string, NodePosition> {
	const layout = new Map<string, NodePosition>()
	for (const node of nodes) {
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
	if (graph.triggers.some((trigger) => trigger.id === connection.source)) {
		return connection.sourceHandle == null
	}

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
	lastRun,
	runStatuses,
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
		buildInitialNodes(graph, layout, descriptors, connectorErrors, runStatuses),
	)
	const [edges, setEdges] = useEdgesState(buildInitialEdges(graph))
	const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null)
	const flowRef = useRef<ReactFlowInstance | null>(null)
	const [openConnectorId, setOpenConnectorId] = useState<string | null>(null)
	const [openTriggerId, setOpenTriggerId] = useState<string | null>(null)
	const [paneMenuKey, setPaneMenuKey] = useState(0)
	const [paneMenuPosition, setPaneMenuPosition] = useState<NodePosition>({
		x: 0,
		y: 0,
	})
	const paneMenuTriggerRef = useRef<HTMLSpanElement | null>(null)

	const nodesRef = useRef(nodes)
	nodesRef.current = nodes
	const edgesRef = useRef(edges)
	edgesRef.current = edges

	const catalogue = useMemo(() => [...descriptors.values()], [descriptors])

	useEffect(() => {
		setNodes((current) =>
			current.map((node) => {
				const data = node.data as ConnectorNodeData | TriggerNodeData
				const errors = connectorErrors.get(node.id) ?? []
				const runStatus = runStatuses.get(node.id) ?? null
				if (
					data.errors === errors &&
					(data as ConnectorNodeData).runStatus === runStatus
				) {
					return node
				}
				return { ...node, data: { ...data, errors, runStatus } }
			}),
		)
	}, [connectorErrors, runStatuses, setNodes])

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
			const applicable = changes
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

	const handleDeleteEdge = useCallback(
		(edgeId: string) => {
			handleEdgesChange([{ id: edgeId, type: 'remove' }])
		},
		[handleEdgesChange],
	)

	const handleConnect = useCallback(
		(connection: Connection) => {
			const nextEdges = addEdge(
				{ ...connection, type: 'deletable' },
				edgesRef.current,
			)
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
			explicitPosition?: NodePosition,
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
				...currentGraph,
				connectors: [...currentGraph.connectors, newConnector],
				edges: [
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
			const position =
				explicitPosition ?? nextNodePosition(sourcePosition, siblingIndex)

			const data: ConnectorNodeData = {
				label: descriptor.label,
				branches: descriptor.branches,
				errors: [],
				runStatus: null,
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
			setOpenTriggerId(null)
			setOpenConnectorId(newId)
			flowRef.current?.setCenter(
				position.x + NODE_CENTER_X,
				position.y + NODE_CENTER_Y,
				CENTER_ON_NODE,
			)
		},
		[onChange, setNodes, setEdges],
	)

	const handleRequestDelete = useCallback((connectorId: string) => {
		setPendingDeleteId(connectorId)
	}, [])

	const handleDeleteTrigger = useCallback(
		(triggerId: string) => {
			const currentGraph = buildGraph(nodesRef.current, edgesRef.current)
			const nextGraph = removeTrigger(currentGraph, triggerId)
			const nextNodes = nodesRef.current.filter((node) => node.id !== triggerId)
			const nextEdges = buildInitialEdges(nextGraph)

			nodesRef.current = nextNodes
			edgesRef.current = nextEdges
			setNodes(nextNodes)
			setEdges(nextEdges)
			onChange(nextGraph, buildLayout(nextNodes))
			setOpenTriggerId((current) => (current === triggerId ? null : current))
		},
		[onChange, setNodes, setEdges],
	)

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
			setOpenTriggerId(node.id)
			return
		}
		if (node.type !== 'connector') return
		setOpenTriggerId(null)
		setOpenConnectorId(node.id)
	}, [])

	const handlePaneContextMenu = useCallback(
		(event: ReactMouseEvent | MouseEvent) => {
			event.preventDefault()
			const instance = flowRef.current
			const position = instance
				? instance.screenToFlowPosition({
						x: event.clientX,
						y: event.clientY,
					})
				: { x: event.clientX, y: event.clientY }
			setPaneMenuPosition(position)
			paneMenuTriggerRef.current?.dispatchEvent(
				new MouseEvent('contextmenu', {
					bubbles: true,
					cancelable: true,
					clientX: event.clientX,
					clientY: event.clientY,
				}),
			)
		},
		[],
	)

	const handleAddConnectorFromPane = useCallback(
		(descriptor: Schemas.ConnectorDescriptorResponse) => {
			const currentGraph = buildGraph(nodesRef.current, edgesRef.current)
			const newId = nextConnectorId(currentGraph)
			const newConnector: Schemas.PlacedConnectorDto = {
				id: newId,
				kind: descriptor.kind,
				version: descriptor.version,
				config: {},
			}
			const data: ConnectorNodeData = {
				label: descriptor.label,
				branches: descriptor.branches,
				errors: [],
				runStatus: null,
				connector: newConnector,
			}
			const nextNodes = [
				...nodesRef.current,
				{
					id: newId,
					type: 'connector',
					position: paneMenuPosition,
					data,
				},
			]

			nodesRef.current = nextNodes
			setNodes(nextNodes)
			onChange(buildGraph(nextNodes, edgesRef.current), buildLayout(nextNodes))
			setOpenTriggerId(null)
			setOpenConnectorId(newId)
			setPaneMenuKey((key) => key + 1)
		},
		[onChange, paneMenuPosition, setNodes],
	)

	const handleAddTriggerFromPane = useCallback(
		(kind: Schemas.TriggerKindDto) => {
			const currentGraph = buildGraph(nodesRef.current, edgesRef.current)
			const newId = nextTriggerId(currentGraph)
			const trigger: Schemas.PlacedTriggerDto = { id: newId, kind }
			const nextNodes = [
				...nodesRef.current,
				{
					id: newId,
					type: 'trigger',
					position: paneMenuPosition,
					data: { trigger, errors: [] } satisfies TriggerNodeData,
				},
			]

			nodesRef.current = nextNodes
			setNodes(nextNodes)
			onChange(buildGraph(nextNodes, edgesRef.current), buildLayout(nextNodes))
			setOpenConnectorId(null)
			setOpenTriggerId(kind === 'Manual' ? null : newId)
			setPaneMenuKey((key) => key + 1)
		},
		[onChange, paneMenuPosition, setNodes],
	)

	const handleTriggerKindChange = useCallback(
		(triggerId: string, kind: Schemas.TriggerKindDto) => {
			const nextNodes = nodesRef.current.map((node) => {
				if (node.id !== triggerId) return node
				const data = node.data as TriggerNodeData
				return {
					...node,
					data: { ...data, trigger: { ...data.trigger, kind } },
				}
			})
			nodesRef.current = nextNodes
			setNodes(nextNodes)
			onChange(buildGraph(nextNodes, edgesRef.current), buildLayout(nextNodes))
		},
		[onChange, setNodes],
	)

	const handleFitViewFromPane = useCallback(() => {
		flowRef.current?.fitView(FIT_VIEW_OPTIONS)
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
			onRequestDeleteTrigger: handleDeleteTrigger,
			onDeleteEdge: handleDeleteEdge,
		}),
		[
			catalogue,
			handleAddNode,
			handleRequestDelete,
			handleDeleteTrigger,
			handleDeleteEdge,
		],
	)

	const openNode = openConnectorId
		? nodes.find((node) => node.id === openConnectorId)
		: undefined
	const openConnectorData = openNode?.data as ConnectorNodeData | undefined
	const openTriggerData = openTriggerId
		? (nodes.find((node) => node.id === openTriggerId)?.data as
				| TriggerNodeData
				| undefined)
		: undefined
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

		const descriptor = descriptors.get(data.connector.kind)
		const mirrored = descriptor?.output_mirrors_field
		if (mirrored) return data.connector.config[mirrored] ?? {}

		return descriptor?.output_example ?? null
	}

	const upstreamIds = openConnectorId
		? upstreamConnectorIds(buildGraph(nodes, edges), openConnectorId)
		: []

	const subscribedEventNames = nodes
		.filter((node) => node.type === 'trigger')
		.flatMap((node) =>
			triggerEventNames((node.data as TriggerNodeData).trigger),
		)
	const triggerExample = resolveTriggerExample(events, [
		...new Set(subscribedEventNames),
	])
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
								edgeTypes={EDGE_TYPES}
								onNodesChange={handleNodesChange}
								onEdgesChange={handleEdgesChange}
								onConnect={handleConnect}
								onNodeDragStop={handleNodeDragStop}
								onNodeClick={handleNodeClick}
								onPaneClick={() => {
									setOpenConnectorId(null)
									setOpenTriggerId(null)
								}}
								onPaneContextMenu={handlePaneContextMenu}
								isValidConnection={validateConnection}
								autoPanOnNodeDrag={false}
								fitView
								fitViewOptions={FIT_VIEW_OPTIONS}
								onInit={(instance) => {
									flowRef.current = instance
								}}
							>
								<Background />
							</ReactFlow>
						</ReactFlowProvider>
						<ContextMenu key={paneMenuKey}>
							<ContextMenuTrigger asChild>
								<span ref={paneMenuTriggerRef} className="sr-only" />
							</ContextMenuTrigger>
							<ContextMenuContent>
								<ContextMenuSub>
									<ContextMenuSubTrigger>
										Ajouter un connecteur
									</ContextMenuSubTrigger>
									<ContextMenuSubContent className="p-2">
										<ConnectorSearch
											connectors={catalogue}
											onSelect={handleAddConnectorFromPane}
										/>
									</ContextMenuSubContent>
								</ContextMenuSub>
								<ContextMenuSub>
									<ContextMenuSubTrigger>
										Ajouter un déclencheur
									</ContextMenuSubTrigger>
									<ContextMenuSubContent>
										<ContextMenuItem
											onSelect={() => handleAddTriggerFromPane('Manual')}
										>
											Manuel
										</ContextMenuItem>
										<ContextMenuItem
											onSelect={() => handleAddTriggerFromPane({ Events: [] })}
										>
											Sur événement(s)
										</ContextMenuItem>
									</ContextMenuSubContent>
								</ContextMenuSub>
								<ContextMenuItem onSelect={handleFitViewFromPane}>
									Cadrer le graphe
								</ContextMenuItem>
							</ContextMenuContent>
						</ContextMenu>
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
					{openTriggerId && openTriggerData ? (
						<TriggerConfigPanel
							trigger={openTriggerData.trigger}
							events={events}
							errors={openTriggerData.errors}
							onClose={() => setOpenTriggerId(null)}
							onChange={(kind) => handleTriggerKindChange(openTriggerId, kind)}
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
