import { Handle, type NodeProps, Position } from '@xyflow/react'
import { TriangleAlert, Zap } from 'lucide-react'
import { cn } from '#/lib/utils'

export interface TriggerNodeData extends Record<string, unknown> {
	hasEvent: boolean
}

export function TriggerNode({ data }: NodeProps & { data: TriggerNodeData }) {
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
			<Handle
				type="source"
				position={Position.Right}
				isConnectableStart={false}
			/>
		</div>
	)
}
