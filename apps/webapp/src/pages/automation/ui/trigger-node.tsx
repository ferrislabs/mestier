import type { NodeProps } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import { TRIGGER_APPEARANCE } from '#/pages/automation/lib/connector-appearance'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'
import { AutomationNodeCard } from '#/pages/automation/ui/automation-node-card'
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

const NO_ISSUES: readonly string[] = []

const OUTPUTS = [{ id: null, open: true }]

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
		<AutomationNodeCard
			label="Déclencheur"
			subtitle={triggerSummary(data.trigger)}
			icon={TRIGGER_APPEARANCE.icon}
			hue={TRIGGER_APPEARANCE.hue}
			trigger
			status={null}
			needsSetup={warns && !hasError}
			disabled={false}
			bindingIssues={NO_ISSUES}
			selected={selected ?? false}
			hasInput={false}
			outputs={OUTPUTS}
			errorLabel={hasError ? `Erreur sur ${id}` : null}
			deleteLabel={`Supprimer le déclencheur ${id}`}
			onDelete={() => actions.onRequestDeleteTrigger(id)}
			renderAdd={() => (
				<AddNodeButton
					ariaLabel={`Ajouter un connecteur après le déclencheur ${id}`}
					catalogue={actions.catalogue}
					onSelect={(connector) => actions.onAddNode(id, null, connector)}
				/>
			)}
			onOpen={() => {}}
			onAddAfter={() => {}}
		/>
	)
}
