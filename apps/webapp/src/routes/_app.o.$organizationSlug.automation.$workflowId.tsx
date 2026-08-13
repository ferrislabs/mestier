import { createFileRoute } from '@tanstack/react-router'
import { WorkflowEditorFeature } from '#/pages/automation/feature/workflow-editor-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automation/$workflowId',
)({
	component: WorkflowEditorPage,
})

function WorkflowEditorPage() {
	const { workflowId } = Route.useParams()
	return <WorkflowEditorFeature workflowId={workflowId} />
}
