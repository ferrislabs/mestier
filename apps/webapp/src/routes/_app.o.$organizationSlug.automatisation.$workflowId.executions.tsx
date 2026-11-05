import { createFileRoute, useParams } from '@tanstack/react-router'
import { WorkflowExecutionsFeature } from '#/pages/automation/feature/workflow-executions-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions',
)({
	component: WorkflowExecutionsPage,
})

function WorkflowExecutionsPage() {
	const { workflowId } = Route.useParams()
	const runId = useParams({
		strict: false,
		select: (params) => params.runId,
	})

	return <WorkflowExecutionsFeature workflowId={workflowId} runId={runId} />
}
