import {
	Background,
	BackgroundVariant,
	Controls,
	type Edge,
	ReactFlow,
	ReactFlowProvider,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useMemo } from 'react'
import type { Schemas } from '#/api/api.client'
import { formatItemCount } from '#/pages/automation/lib/format'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { runNodeAppearance } from '#/pages/automation/lib/run-appearance'
import { type RunView, realEdgeId } from '#/pages/automation/lib/run-view'
import type { ExecutionFlowNodeType } from '#/pages/automation/ui/execution-flow-node'
import { ExecutionFlowNode } from '#/pages/automation/ui/execution-flow-node'
import { ExecutionLegend } from '#/pages/automation/ui/execution-legend'
import '#/pages/automation/ui/automation.css'

const NODE_TYPES = { execution: ExecutionFlowNode }

const FIT_VIEW_OPTIONS = { padding: 0.3, maxZoom: 1 }

const GRID_SIZE = 20

const ARIA_LABELS = {
	'controls.ariaLabel': 'Contrôles du canevas',
	'controls.zoomIn.ariaLabel': 'Zoomer',
	'controls.zoomOut.ariaLabel': 'Dézoomer',
	'controls.fitView.ariaLabel': 'Ajuster la vue',
}

const TRAVERSED_STROKE = 'var(--auto-success)'
const RUNNING_STROKE = 'var(--state-today)'

function ExecutionFlow({
	graph,
	layout,
	descriptors,
	view,
	onOpenNode,
}: {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	view: RunView
	onOpenNode: (nodeId: string) => void
}) {
	const nodes = useMemo<ExecutionFlowNodeType[]>(
		() =>
			view.nodes.map((node) => {
				const appearance = runNodeAppearance(node)
				const branches =
					node.connector !== null
						? (descriptors.get(node.connector.kind)?.branches ?? [])
						: []
				return {
					id: node.id,
					type: 'execution',
					position: layout.get(node.id) ?? { x: 0, y: 0 },
					data: {
						label: node.label,
						subtitle: node.subtitle,
						icon: appearance.icon,
						hue: appearance.hue,
						trigger: node.kind === 'trigger',
						outputs:
							branches.length === 0
								? [{ id: null, open: false }]
								: branches.map((branch) => ({
										id: branch,
										label: branch,
										open: false,
									})),
						execution: { status: node.status, note: node.note },
						onOpen: () => onOpenNode(node.id),
					},
				}
			}),
		[view, layout, descriptors, onOpenNode],
	)

	const edges = useMemo<Edge[]>(
		() =>
			graph.edges.map((edge) => {
				const base = {
					id: realEdgeId(edge),
					source: edge.from,
					target: edge.to,
					sourceHandle: edge.branch ?? undefined,
				}
				if (!view.traversedEdges.has(base.id))
					return {
						...base,
						style: { strokeDasharray: '5 5', opacity: 0.45 },
					}
				const intoRunning =
					view.nodes.find((node) => node.id === edge.to)?.status === 'running'
				const stroke = intoRunning ? RUNNING_STROKE : TRAVERSED_STROKE
				const count = view.edgeCounts.get(base.id)
				return {
					...base,
					animated: intoRunning,
					style: { stroke, strokeWidth: 2.5 },
					...(count === undefined
						? {}
						: {
								label: formatItemCount(count),
								labelStyle: { fill: stroke, fontSize: 11, fontWeight: 600 },
								labelBgStyle: { fill: 'var(--background)' },
								labelBgPadding: [6, 3] as [number, number],
								labelBgBorderRadius: 4,
							}),
				}
			}),
		[graph, view],
	)

	return (
		<ReactFlow
			className="automation-flow"
			nodes={nodes}
			edges={edges}
			nodeTypes={NODE_TYPES}
			onNodeClick={(_, node) => onOpenNode(node.id)}
			nodesDraggable={false}
			nodesConnectable={false}
			elementsSelectable={false}
			edgesFocusable={false}
			nodesFocusable={false}
			zoomOnDoubleClick={false}
			fitView
			fitViewOptions={FIT_VIEW_OPTIONS}
			minZoom={0.25}
			maxZoom={1.5}
			ariaLabelConfig={ARIA_LABELS}
		>
			<Background variant={BackgroundVariant.Dots} gap={GRID_SIZE} size={1.6} />
			<Controls
				position="bottom-left"
				showInteractive={false}
				fitViewOptions={{ ...FIT_VIEW_OPTIONS, duration: 300 }}
			/>
		</ReactFlow>
	)
}

export function ExecutionCanvas(props: {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	view: RunView
	onOpenNode: (nodeId: string) => void
}) {
	return (
		<div className="relative h-full w-full">
			<ReactFlowProvider>
				<ExecutionFlow {...props} />
			</ReactFlowProvider>
			<div className="pointer-events-none absolute top-3 left-3 z-10">
				<ExecutionLegend />
			</div>
		</div>
	)
}
