import { createFileRoute } from '@tanstack/react-router'
import { RunInspectorFeature } from '#/pages/automation/feature/run-inspector-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions/$runId',
)({
	component: RunDetailPage,
})

function RunDetailPage() {
	const { workflowId, runId } = Route.useParams()

	return <RunInspectorFeature workflowId={workflowId} runId={runId} />
}
