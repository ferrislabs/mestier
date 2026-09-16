import { Handle, type NodeProps, Position } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import { StatusBadge } from '#/components/ui/surface'
import { cn } from '#/lib/utils'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface RunConnectorNodeData extends Record<string, unknown> {
	label: string
	branches: Schemas.BranchDto[]
	status: string | null
}

export function RunConnectorNode({
	data,
}: NodeProps & { data: RunConnectorNodeData }) {
	const hasFailed = data.status === 'failed'

	return (
		<div
			className={cn(
				'min-w-40 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm',
				hasFailed && 'border-destructive',
			)}
		>
			<Handle type="target" position={Position.Left} />
			<div className="flex items-center justify-between gap-2">
				<span className="font-medium">{data.label}</span>
				<StatusBadge
					tone={
						data.status
							? (RUN_STATUS_TONE[data.status] ?? 'neutral')
							: 'neutral'
					}
				>
					{data.status
						? (RUN_STATUS_LABEL[data.status] ?? data.status)
						: 'Non exécuté'}
				</StatusBadge>
			</div>
			{data.branches.length === 0 ? (
				<Handle type="source" position={Position.Right} />
			) : (
				<div className="mt-2 flex flex-col gap-2">
					{data.branches.map((branch, index) => (
						<div
							key={branch}
							className="relative flex items-center justify-end pr-2 text-xs text-muted-foreground"
						>
							{branch}
							<Handle
								type="source"
								position={Position.Right}
								id={branch}
								style={{
									top: `${((index + 1) / (data.branches.length + 1)) * 100}%`,
								}}
							/>
						</div>
					))}
				</div>
			)}
		</div>
	)
}
