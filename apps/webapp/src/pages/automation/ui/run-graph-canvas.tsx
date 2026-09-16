import {
	Background,
	type Edge,
	type Node,
	ReactFlow,
	ReactFlowProvider,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useMemo } from 'react'
import type { Schemas } from '#/api/api.client'
import type { NodePosition } from '#/pages/automation/lib/graph'
import {
	RunConnectorNode,
	type RunConnectorNodeData,
} from '#/pages/automation/ui/run-connector-node'

const NODE_TYPES = { connector: RunConnectorNode }

export interface RunGraphCanvasProps {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorStatuses: Map<string, string>
}

function realEdgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

export function RunGraphCanvas({
	graph,
	layout,
	descriptors,
	connectorStatuses,
}: RunGraphCanvasProps) {
	const nodes: Node[] = useMemo(
		() =>
			graph.connectors.map((connector) => {
				const descriptor = descriptors.get(connector.kind)
				const data: RunConnectorNodeData = {
					label: descriptor?.label ?? connector.id,
					branches: descriptor?.branches ?? [],
					status: connectorStatuses.get(connector.id) ?? null,
				}
				return {
					id: connector.id,
					type: 'connector',
					position: layout.get(connector.id) ?? { x: 0, y: 0 },
					data,
					draggable: false,
				}
			}),
		[graph, layout, descriptors, connectorStatuses],
	)

	const edges: Edge[] = useMemo(
		() =>
			graph.edges.map((edge) => ({
				id: realEdgeId(edge),
				source: edge.from,
				target: edge.to,
				sourceHandle: edge.branch ?? undefined,
			})),
		[graph],
	)

	return (
		<div className="min-h-0 min-w-0 flex-1">
			<ReactFlowProvider>
				<ReactFlow
					nodes={nodes}
					edges={edges}
					nodeTypes={NODE_TYPES}
					nodesDraggable={false}
					nodesConnectable={false}
					elementsSelectable={false}
					fitView
				>
					<Background />
				</ReactFlow>
			</ReactFlowProvider>
		</div>
	)
}
