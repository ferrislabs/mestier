import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationRuns,
	useAutomationWorkflows,
	useCreateWorkflow,
	useDeleteWorkflow,
	useUpdateWorkflow,
	type Workflow,
} from '#/hooks/use-automation'
import { latestRunByWorkflow } from '#/pages/automation/lib/workflow-runs'
import {
	EMPTY_WORKFLOW_FORM,
	type WorkflowFormValues,
} from '#/pages/automation/types'
import {
	AutomationWorkflowsList,
	type WorkflowRow,
} from '#/pages/automation/ui/automation-workflows-list'
import { WorkflowEditorDialog } from '#/pages/automation/ui/workflow-editor-dialog'

export function AutomationWorkflowsFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<AutomationWorkflowsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

interface Draft {
	id: string | null
	values: WorkflowFormValues
}

function AutomationWorkflowsWorkspace({
	organizationId,
	organizationName,
	organizationSlug,
}: {
	organizationId: string
	organizationName: string
	organizationSlug: string
}) {
	const [draft, setDraft] = useState<Draft | null>(null)

	const workflows = useAutomationWorkflows(organizationId)
	const runs = useAutomationRuns(organizationId)
	const createWorkflow = useCreateWorkflow(organizationId)
	const updateWorkflow = useUpdateWorkflow()
	const deleteWorkflow = useDeleteWorkflow()

	const items: Workflow[] = workflows.data?.data ?? []
	const latestRuns = latestRunByWorkflow(runs.data?.data ?? [])

	const rows: WorkflowRow[] = items.map((workflow) => ({
		id: workflow.id,
		name: workflow.name,
		description: workflow.description ?? null,
		enabled: workflow.enabled,
		lastRun: latestRuns.get(workflow.id) ?? null,
	}))

	const isPending = createWorkflow.isPending || updateWorkflow.isPending

	const error =
		workflows.error ??
		runs.error ??
		createWorkflow.error ??
		updateWorkflow.error ??
		deleteWorkflow.error

	const submitDraft = async () => {
		if (!draft) return

		if (draft.id === null) {
			await createWorkflow.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
				},
			})
		} else {
			await updateWorkflow.mutateAsync({
				path: { organization_id: organizationId, workflow_id: draft.id },
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
				},
			})
		}

		setDraft(null)
	}

	return (
		<>
			<AutomationWorkflowsList
				organizationName={organizationName}
				organizationSlug={organizationSlug}
				workflows={rows}
				isLoading={workflows.isLoading}
				error={error?.message ?? null}
				onCreate={() => setDraft({ id: null, values: EMPTY_WORKFLOW_FORM })}
				onRename={(row) =>
					setDraft({
						id: row.id,
						values: { name: row.name, description: row.description ?? '' },
					})
				}
				onToggleEnabled={(row) =>
					void updateWorkflow.mutateAsync({
						path: { organization_id: organizationId, workflow_id: row.id },
						body: { enabled: !row.enabled },
					})
				}
				onDelete={(row) =>
					void deleteWorkflow.mutateAsync({
						path: { organization_id: organizationId, workflow_id: row.id },
					})
				}
			/>

			<WorkflowEditorDialog
				open={draft !== null}
				editingName={
					draft?.id
						? (rows.find((row) => row.id === draft.id)?.name ?? null)
						: null
				}
				values={draft?.values ?? EMPTY_WORKFLOW_FORM}
				isPending={isPending}
				error={error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setDraft(null)
				}}
				onValuesChange={(patch) =>
					setDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onSubmit={() => void submitDraft()}
			/>
		</>
	)
}
