import { createFileRoute } from '@tanstack/react-router'
import { WorkflowRunsFeature } from '#/pages/automation/feature/workflow-runs-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automation/$workflowId/runs',
)({
	component: WorkflowRunsPage,
})

function WorkflowRunsPage() {
	const { workflowId } = Route.useParams()
	return <WorkflowRunsFeature workflowId={workflowId} />
}
