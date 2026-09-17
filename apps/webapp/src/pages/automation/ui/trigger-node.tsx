import { Handle, type NodeProps, Position } from '@xyflow/react'
import { Trash2, TriangleAlert, Zap } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { cn } from '#/lib/utils'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'
import { useWorkflowCanvasActions } from '#/pages/automation/ui/workflow-canvas-context'

export interface TriggerNodeData extends Record<string, unknown> {
	trigger: Schemas.PlacedTriggerDto
	errors: ConnectorValidationError[]
}

export function triggerEventNames(trigger: Schemas.PlacedTriggerDto): string[] {
	return trigger.kind === 'Manual' ? [] : trigger.kind.Events
}

export function isManualTrigger(trigger: Schemas.PlacedTriggerDto): boolean {
	return trigger.kind === 'Manual'
}

function triggerSummary(trigger: Schemas.PlacedTriggerDto): string {
	if (isManualTrigger(trigger)) return 'Déclenchement manuel'

	const names = triggerEventNames(trigger)
	if (names.length === 0) return 'Aucun événement configuré'
	if (names.length === 1) return names[0]
	return `${names.length} événements`
}

export function TriggerNode({
	id,
	data,
	selected,
}: NodeProps & { data: TriggerNodeData }) {
	const actions = useWorkflowCanvasActions()
	const hasError = data.errors.length > 0
	const warns =
		!isManualTrigger(data.trigger) &&
		triggerEventNames(data.trigger).length === 0

	return (
		<div
			className={cn(
				'min-w-40 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm',
				selected && 'ring-2 ring-primary',
				hasError && 'border-destructive',
				!hasError && warns && 'border-amber-500',
			)}
		>
			<div className="flex items-center justify-between gap-2">
				<span className="flex items-center gap-1.5 font-medium">
					<Zap className="size-3.5" />
					Déclencheur
				</span>
				<div className="flex items-center gap-1">
					{hasError ? (
						<Badge variant="destructive" aria-label={`Erreur sur ${id}`}>
							!
						</Badge>
					) : null}
					<button
						type="button"
						aria-label={`Supprimer le déclencheur ${id}`}
						className="nodrag flex size-5 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-destructive"
						onClick={(event) => {
							event.stopPropagation()
							actions.onRequestDeleteTrigger(id)
						}}
					>
						<Trash2 className="size-3.5" />
					</button>
				</div>
			</div>
			<div
				className={cn(
					'mt-1 flex items-center gap-1 text-xs text-muted-foreground',
					warns && 'text-amber-700',
				)}
			>
				{warns ? <TriangleAlert className="size-3.5" /> : null}
				{triggerSummary(data.trigger)}
			</div>
			<div className="relative mt-2 flex items-center justify-end gap-1 pr-2">
				<AddNodeButton
					ariaLabel={`Ajouter un connecteur après le déclencheur ${id}`}
					catalogue={actions.catalogue}
					onSelect={(connector) => actions.onAddNode(id, null, connector)}
				/>
				<Handle type="source" position={Position.Right} />
			</div>
		</div>
	)
}
