import { createFileRoute, Outlet } from '@tanstack/react-router'
import { WorkflowCanvasFeature } from '#/pages/automation/feature/workflow-canvas-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId',
)({
	component: WorkflowEditorPage,
})

function WorkflowEditorPage() {
	const { workflowId } = Route.useParams()

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowCanvasFeature workflowId={workflowId} />
			<Outlet />
		</div>
	)
}
