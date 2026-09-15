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
import { useCallback, useRef } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { rootConnectorIds } from '#/pages/automation/lib/graph'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	ConnectorNode,
	type ConnectorNodeData,
} from '#/pages/automation/ui/connector-node'
import { TriggerNode } from '#/pages/automation/ui/trigger-node'

export const TRIGGER_NODE_ID = '__trigger__'
const TRIGGER_X_OFFSET = 220

const NODE_TYPES = {
	connector: ConnectorNode,
	trigger: TriggerNode,
}

export interface WorkflowCanvasProps {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors: Map<string, ConnectorValidationError[]>
	hasTriggerEvent: boolean
	isDirty: boolean
	isSaving: boolean
	onChange: (graph: Schemas.GraphDto, layout: Map<string, NodePosition>) => void
	onSave: () => void
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
	hasTriggerEvent: boolean,
): Node[] {
	const triggerNode: Node = {
		id: TRIGGER_NODE_ID,
		type: 'trigger',
		position: triggerPosition(graph, layout),
		data: { hasEvent: hasTriggerEvent },
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

export function isBranchDeclared(
	connection: Pick<Connection, 'source' | 'sourceHandle'>,
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
	hasTriggerEvent,
	isDirty,
	isSaving,
	onChange,
	onSave,
}: WorkflowCanvasProps) {
	const [nodes, , onNodesChangeInternal] = useNodesState(() =>
		buildInitialNodes(
			graph,
			layout,
			descriptors,
			connectorErrors,
			hasTriggerEvent,
		),
	)
	const [edges, setEdges] = useEdgesState(() => buildInitialEdges(graph))

	const nodesRef = useRef(nodes)
	nodesRef.current = nodes
	const edgesRef = useRef(edges)
	edgesRef.current = edges

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
			isBranchDeclared(connection, graph, descriptors),
		[graph, descriptors],
	)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<div className="flex items-center justify-between border-b px-4 py-2">
				<span className="text-xs text-muted-foreground">
					{isDirty ? 'Modifications non enregistrées' : 'Enregistré'}
				</span>
				<Button size="sm" onClick={onSave} disabled={isSaving}>
					{isSaving ? 'Enregistrement…' : 'Enregistrer'}
				</Button>
			</div>
			<div className="min-h-0 flex-1">
				<ReactFlowProvider>
					<ReactFlow
						nodes={nodes}
						edges={edges}
						nodeTypes={NODE_TYPES}
						onNodesChange={handleNodesChange}
						onEdgesChange={handleEdgesChange}
						onConnect={handleConnect}
						onNodeDragStop={handleNodeDragStop}
						isValidConnection={validateConnection}
						autoPanOnNodeDrag={false}
						fitView={false}
					>
						<Background />
					</ReactFlow>
				</ReactFlowProvider>
			</div>
		</div>
	)
}
