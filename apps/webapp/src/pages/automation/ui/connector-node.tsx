import { Handle, type NodeProps, Position } from '@xyflow/react'
import { Trash2 } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { cn } from '#/lib/utils'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'
import { useWorkflowCanvasActions } from '#/pages/automation/ui/workflow-canvas-context'

export interface ConnectorNodeData extends Record<string, unknown> {
	label: string
	branches: Schemas.BranchDto[]
	errors: ConnectorValidationError[]
	connector: Schemas.PlacedConnectorDto
}

export function ConnectorNode({
	id,
	data,
	selected,
}: NodeProps & { data: ConnectorNodeData }) {
	const actions = useWorkflowCanvasActions()
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
				<div className="flex items-center gap-1">
					{hasError ? (
						<Badge
							variant="destructive"
							aria-label={`Erreur sur ${data.label}`}
						>
							!
						</Badge>
					) : null}
					<button
						type="button"
						aria-label={`Supprimer ${data.label}`}
						className="nodrag flex size-5 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-destructive"
						onClick={(event) => {
							event.stopPropagation()
							actions.onRequestDelete(id)
						}}
					>
						<Trash2 className="size-3.5" />
					</button>
				</div>
			</div>
			{data.branches.length === 0 ? (
				<div className="relative mt-2 flex items-center justify-end gap-1 pr-2">
					<AddNodeButton
						ariaLabel={`Ajouter un connecteur après ${data.label}`}
						catalogue={actions.catalogue}
						onSelect={(connector) => actions.onAddNode(id, null, connector)}
					/>
					<Handle type="source" position={Position.Right} />
				</div>
			) : (
				<div className="mt-2 flex flex-col gap-2">
					{data.branches.map((branch, index) => (
						<div
							key={branch}
							className="relative flex items-center justify-end gap-1 pr-2 text-xs text-muted-foreground"
						>
							{branch}
							<AddNodeButton
								ariaLabel={`Ajouter un connecteur sur la branche ${branch}`}
								catalogue={actions.catalogue}
								onSelect={(connector) =>
									actions.onAddNode(id, branch, connector)
								}
							/>
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
