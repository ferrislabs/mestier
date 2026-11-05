import {
	createFileRoute,
	Outlet,
	useChildMatches,
} from '@tanstack/react-router'
import { WorkflowCanvasFeature } from '#/pages/automation/feature/workflow-canvas-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId',
)({
	component: WorkflowEditorPage,
})

function WorkflowEditorPage() {
	const { workflowId } = Route.useParams()
	const childMatches = useChildMatches()

	if (childMatches.length > 0) return <Outlet />

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowCanvasFeature workflowId={workflowId} />
		</div>
	)
}
