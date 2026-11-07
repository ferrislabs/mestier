import { createFileRoute } from '@tanstack/react-router'
import { WorkflowPageFeature } from '#/pages/automation/feature/workflow-page-feature'
import { parseWorkflowSearch } from '#/pages/automation/lib/automation-paths'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/workflows/$workflowId',
)({
	validateSearch: parseWorkflowSearch,
	component: WorkflowPage,
})

function WorkflowPage() {
	const { workflowId } = Route.useParams()
	const search = Route.useSearch()

	return (
		<WorkflowPageFeature
			workflowId={workflowId}
			view={search.vue === 'executions' ? 'executions' : 'editor'}
			executionNumber={search.execution}
		/>
	)
}
