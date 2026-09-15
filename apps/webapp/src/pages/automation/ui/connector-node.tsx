import { Handle, type NodeProps, Position } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { cn } from '#/lib/utils'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'

export interface ConnectorNodeData extends Record<string, unknown> {
	label: string
	branches: Schemas.BranchDto[]
	errors: ConnectorValidationError[]
	connector: Schemas.PlacedConnectorDto
}

export function ConnectorNode({
	data,
	selected,
}: NodeProps & { data: ConnectorNodeData }) {
	const hasError = data.errors.length > 0

	return (
		<div
			className={cn(
				'min-w-40 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm',
				selected && 'ring-2 ring-primary',
				hasError && 'border-destructive',
			)}
		>
			<Handle type="target" position={Position.Left} />
			<div className="flex items-center justify-between gap-2">
				<span className="font-medium">{data.label}</span>
				{hasError ? (
					<Badge variant="destructive" aria-label={`Erreur sur ${data.label}`}>
						!
					</Badge>
				) : null}
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
