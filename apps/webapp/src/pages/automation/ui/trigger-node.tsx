import { Handle, type NodeProps, Position } from '@xyflow/react'
import { TriangleAlert, Zap } from 'lucide-react'
import { cn } from '#/lib/utils'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'
import { useWorkflowCanvasActions } from '#/pages/automation/ui/workflow-canvas-context'

export interface TriggerNodeData extends Record<string, unknown> {
	hasEvent: boolean
}

export function TriggerNode({
	id,
	data,
}: NodeProps & { data: TriggerNodeData }) {
	const actions = useWorkflowCanvasActions()

	return (
		<div
			className={cn(
				'min-w-40 rounded-full border-2 border-dashed bg-card px-4 py-2 text-sm',
				!data.hasEvent && 'border-amber-500 text-amber-700',
			)}
		>
			<div className="flex items-center gap-1.5 font-medium">
				<Zap className="size-3.5" />
				Déclencheur
			</div>
			{!data.hasEvent ? (
				<div className="mt-1 flex items-center gap-1 text-xs">
					<TriangleAlert className="size-3.5" />
					Aucun événement configuré
				</div>
			) : null}
			<div className="relative mt-1 flex items-center justify-end gap-1 pr-2">
				<AddNodeButton
					ariaLabel="Ajouter le premier connecteur"
					catalogue={actions.catalogue}
					onSelect={(connector) => actions.onAddNode(id, null, connector)}
				/>
				<Handle
					type="source"
					position={Position.Right}
					isConnectableStart={false}
				/>
			</div>
		</div>
	)
}
