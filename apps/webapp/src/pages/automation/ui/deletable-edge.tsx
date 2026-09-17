import {
	BaseEdge,
	EdgeLabelRenderer,
	type EdgeProps,
	getBezierPath,
} from '@xyflow/react'
import { Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useWorkflowCanvasActions } from '#/pages/automation/ui/workflow-canvas-context'

const HOVER_STROKE_WIDTH = 20

export function DeletableEdge({
	id,
	sourceX,
	sourceY,
	targetX,
	targetY,
	sourcePosition,
	targetPosition,
	markerEnd,
	style,
	selected,
}: EdgeProps) {
	const actions = useWorkflowCanvasActions()
	const [hovered, setHovered] = useState(false)
	const [path, labelX, labelY] = getBezierPath({
		sourceX,
		sourceY,
		sourcePosition,
		targetX,
		targetY,
		targetPosition,
	})

	return (
		<>
			<BaseEdge
				id={id}
				path={path}
				markerEnd={markerEnd}
				style={style}
				interactionWidth={0}
			/>
			{/* biome-ignore lint/a11y/noStaticElementInteractions: widening the hit
			area of a decorative path; the control it reveals is the labelled
			button below, which is also reachable by selecting the edge. */}
			<path
				d={path}
				fill="none"
				stroke="transparent"
				strokeWidth={HOVER_STROKE_WIDTH}
				className="react-flow__edge-interaction"
				onMouseEnter={() => setHovered(true)}
				onMouseLeave={() => setHovered(false)}
			/>
			{hovered || selected ? (
				<EdgeLabelRenderer>
					<button
						type="button"
						aria-label={`Supprimer la liaison ${id}`}
						className="nodrag nopan pointer-events-auto absolute flex size-6 items-center justify-center rounded-full border bg-card text-muted-foreground shadow-sm hover:bg-accent hover:text-destructive"
						style={{
							transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
						}}
						onMouseEnter={() => setHovered(true)}
						onMouseLeave={() => setHovered(false)}
						onClick={(event) => {
							event.stopPropagation()
							actions.onDeleteEdge(id)
						}}
					>
						<Trash2 className="size-3.5" />
					</button>
				</EdgeLabelRenderer>
			) : null}
		</>
	)
}
