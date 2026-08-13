import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useAutomationRuns, useWorkflow } from '#/hooks/use-automation'
import {
	type RunRow,
	WorkflowRunsUI,
} from '#/pages/automation/ui/workflow-runs-ui'

export interface WorkflowRunsFeatureProps {
	workflowId: string
}

export function WorkflowRunsFeature({ workflowId }: WorkflowRunsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowRuns
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
		/>
	)
}

interface WorkflowRunsProps {
	organizationId: string
	organizationName: string
	organizationSlug: string
	workflowId: string
}

function WorkflowRuns({
	organizationId,
	organizationName,
	organizationSlug,
	workflowId,
}: WorkflowRunsProps) {
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const runsQuery = useAutomationRuns(organizationId)

	// The backend has no per-workflow runs endpoint — the org-wide list is
	// filtered here. It is already most-recent-first, and filtering keeps
	// that order.
	const runs = (runsQuery.data?.data ?? []).filter(
		(run) => run.workflow_id === workflowId,
	)

	const rows: RunRow[] = runs.map((run) => ({
		id: run.id,
		status: run.status,
		startedAt: run.started_at ?? null,
		finishedAt: run.finished_at ?? null,
		error: run.error ?? null,
	}))

	return (
		<WorkflowRunsUI
			organizationName={organizationName}
			organizationSlug={organizationSlug}
			workflowId={workflowId}
			workflowName={workflowQuery.data?.data.name ?? null}
			isLoading={runsQuery.isLoading || workflowQuery.isLoading}
			error={runsQuery.error?.message ?? workflowQuery.error?.message ?? null}
			runs={rows}
		/>
	)
}
