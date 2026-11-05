import type { NodeProps } from '@xyflow/react'
import type { Schemas } from '#/api/api.client'
import { connectorAppearance } from '#/pages/automation/lib/connector-appearance'
import { executionFromRunStatus } from '#/pages/automation/lib/execution-status'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import { AddNodeButton } from '#/pages/automation/ui/add-node-button'
import { AutomationNodeCard } from '#/pages/automation/ui/automation-node-card'
import { useWorkflowCanvasActions } from '#/pages/automation/ui/workflow-canvas-context'

export interface ConnectorNodeData extends Record<string, unknown> {
	label: string
	family?: string
	branches: Schemas.BranchDto[]
	errors: ConnectorValidationError[]
	runStatus: string | null
	connector: Schemas.PlacedConnectorDto
}

const NO_ISSUES: readonly string[] = []

export function ConnectorNode({
	id,
	data,
	selected,
}: NodeProps & { data: ConnectorNodeData }) {
	const actions = useWorkflowCanvasActions()
	const appearance = connectorAppearance(data.family)
	const runStatus = executionFromRunStatus(data.runStatus)
	const outputs =
		data.branches.length === 0
			? [{ id: null, open: true }]
			: data.branches.map((branch) => ({
					id: branch,
					label: branch,
					open: true,
				}))

	return (
		<AutomationNodeCard
			label={data.label}
			subtitle={data.family ?? data.connector.kind}
			icon={appearance.icon}
			hue={appearance.hue}
			trigger={false}
			status={null}
			needsSetup={false}
			disabled={false}
			bindingIssues={NO_ISSUES}
			selected={selected ?? false}
			hasInput
			outputs={outputs}
			execution={runStatus ? { status: runStatus, note: '' } : null}
			errorLabel={data.errors.length > 0 ? `Erreur sur ${data.label}` : null}
			deleteLabel={`Supprimer ${data.label}`}
			onDelete={() => actions.onRequestDelete(id)}
			renderAdd={(output) => {
				const branch = data.branches.find((item) => item === output.id) ?? null
				return (
					<AddNodeButton
						ariaLabel={
							branch === null
								? `Ajouter un connecteur après ${data.label}`
								: `Ajouter un connecteur sur la branche ${branch}`
						}
						catalogue={actions.catalogue}
						onSelect={(connector) => actions.onAddNode(id, branch, connector)}
					/>
				)
			}}
			onOpen={() => {}}
			onAddAfter={() => {}}
		/>
	)
}
