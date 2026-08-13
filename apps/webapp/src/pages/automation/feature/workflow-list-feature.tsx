import { useForm } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
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
import { buildOrgPath } from '#/modules/org-path'
import {
	type WorkflowFormValues,
	WorkflowListUI,
	type WorkflowRow,
} from '#/pages/automation/ui/workflow-list-ui'

export function WorkflowListFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowDirectory
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

interface WorkflowDirectoryProps {
	organizationId: string
	organizationName: string
	organizationSlug: string
}

function WorkflowDirectory({
	organizationId,
	organizationName,
	organizationSlug,
}: WorkflowDirectoryProps) {
	const navigate = useNavigate()
	const workflowsQuery = useAutomationWorkflows(organizationId)
	const runsQuery = useAutomationRuns(organizationId)
	const createWorkflow = useCreateWorkflow(organizationId)
	const updateWorkflow = useUpdateWorkflow()
	const deleteWorkflow = useDeleteWorkflow()

	const [togglingId, setTogglingId] = useState<string | null>(null)
	const [deletingId, setDeletingId] = useState<string | null>(null)
	const [createDialogOpen, setCreateDialogOpen] = useState(false)

	const workflowForm = useForm({
		defaultValues: { name: '', description: '' } satisfies WorkflowFormValues,
		onSubmit: async ({ value }) => {
			const created = await createWorkflow.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					description: value.description.trim() || undefined,
				},
			})
			workflowForm.reset()
			// Closed explicitly rather than left to unmount on navigation — the
			// route transition isn't instant, and a modal still visible behind
			// the page that replaces it would be a rough edge, not a feature.
			setCreateDialogOpen(false)
			await navigate({
				to: buildOrgPath(organizationSlug, '/automation/$workflowId'),
				params: { workflowId: created.data.id },
			})
		},
	})

	const workflows = workflowsQuery.data?.data ?? []
	const runs = runsQuery.data?.data ?? []

	// `useAutomationRuns` already returns the org's runs most recent first, so
	// the first match per workflow is its last run — no re-sort needed.
	const lastRunByWorkflowId = new Map<string, (typeof runs)[number]>()
	for (const run of runs) {
		if (!lastRunByWorkflowId.has(run.workflow_id)) {
			lastRunByWorkflowId.set(run.workflow_id, run)
		}
	}

	const rows: WorkflowRow[] = workflows.map((workflow: Workflow) => {
		const lastRun = lastRunByWorkflowId.get(workflow.id) ?? null
		return {
			id: workflow.id,
			name: workflow.name,
			description: workflow.description ?? null,
			enabled: workflow.enabled,
			lastRunStatus: lastRun?.status ?? null,
			lastRunAt: lastRun?.created_at ?? null,
		}
	})

	const isLoading = workflowsQuery.isLoading
	const error =
		workflowsQuery.error ??
		createWorkflow.error ??
		updateWorkflow.error ??
		deleteWorkflow.error

	const handleToggleEnabled = async (workflow: WorkflowRow) => {
		setTogglingId(workflow.id)
		try {
			await updateWorkflow.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflow.id },
				body: { enabled: !workflow.enabled },
			})
		} finally {
			setTogglingId(null)
		}
	}

	const handleDelete = async (workflow: WorkflowRow) => {
		setDeletingId(workflow.id)
		try {
			await deleteWorkflow.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflow.id },
			})
		} finally {
			setDeletingId(null)
		}
	}

	return (
		<workflowForm.Subscribe selector={(state) => state.values}>
			{(formValues) => (
				<WorkflowListUI
					organizationName={organizationName}
					organizationSlug={organizationSlug}
					isLoading={isLoading}
					error={error?.message ?? null}
					workflows={rows}
					createDialogOpen={createDialogOpen}
					onOpenCreateDialog={() => setCreateDialogOpen(true)}
					onCreateDialogOpenChange={(open) => {
						setCreateDialogOpen(open)
						// Closing without submitting (Escape, overlay click, "Annuler")
						// starts the next "Ajouter" from a blank form, not a stale draft.
						if (!open) workflowForm.reset()
					}}
					createForm={{
						values: formValues,
						isPending: createWorkflow.isPending,
						onChange: (patch) => {
							for (const key of Object.keys(
								patch,
							) as (keyof WorkflowFormValues)[]) {
								workflowForm.setFieldValue(key, patch[key] ?? '')
							}
						},
						onSubmit: () => void workflowForm.handleSubmit(),
					}}
					togglingId={togglingId}
					onToggleEnabled={(workflow) => void handleToggleEnabled(workflow)}
					deletingId={deletingId}
					onDelete={(workflow) => void handleDelete(workflow)}
				/>
			)}
		</workflowForm.Subscribe>
	)
}
