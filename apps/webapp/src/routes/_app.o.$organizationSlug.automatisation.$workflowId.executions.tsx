import {
	createFileRoute,
	Outlet,
	useChildMatches,
} from '@tanstack/react-router'
import { WorkflowRunsFeature } from '#/pages/automation/feature/workflow-runs-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions',
)({
	component: WorkflowExecutionsPage,
})

function WorkflowExecutionsPage() {
	const { workflowId } = Route.useParams()
	const childMatches = useChildMatches()

	if (childMatches.length > 0) return <Outlet />

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowRunsFeature workflowId={workflowId} />
		</div>
	)
}
