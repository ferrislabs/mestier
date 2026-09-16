import { useNavigate } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Run,
	useAutomationRuns,
	useWorkflow,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import {
	formatRunDuration,
	runDurationMs,
	runTriggerLabel,
} from '#/pages/automation/lib/workflow-runs'
import type { RunListRow } from '#/pages/automation/ui/workflow-runs-list'
import { WorkflowRunsList } from '#/pages/automation/ui/workflow-runs-list'

export interface WorkflowRunsFeatureProps {
	workflowId: string
}

export function WorkflowRunsFeature({ workflowId }: WorkflowRunsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowRunsWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
		/>
	)
}

function toRow(run: Run): RunListRow {
	return {
		id: run.id,
		status: run.status,
		trigger: runTriggerLabel(run),
		startedAt: run.started_at ?? null,
		finishedAt: run.finished_at ?? null,
		duration: formatRunDuration(runDurationMs(run)),
	}
}

function WorkflowRunsWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
}) {
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const runsQuery = useAutomationRuns(organizationId)
	const navigate = useNavigate()

	const rows: RunListRow[] = (runsQuery.data?.data ?? [])
		.filter((run) => run.workflow_id === workflowId)
		.sort((a, b) => (a.created_at < b.created_at ? 1 : -1))
		.map(toRow)

	const error = runsQuery.error?.message ?? workflowQuery.error?.message ?? null

	function handleOpenRun(row: RunListRow) {
		void navigate({
			to: buildOrgPath(
				organizationSlug,
				'/automatisation/$workflowId/executions/$runId',
			),
			params: { workflowId, runId: row.id },
		})
	}

	function handleBackToEditor() {
		void navigate({
			to: buildOrgPath(organizationSlug, '/automatisation/$workflowId'),
			params: { workflowId },
		})
	}

	return (
		<WorkflowRunsList
			workflowName={workflowQuery.data?.data?.name ?? ''}
			rows={rows}
			isLoading={runsQuery.isLoading || workflowQuery.isLoading}
			error={error}
			onOpenRun={handleOpenRun}
			onBackToEditor={handleBackToEditor}
		/>
	)
}
