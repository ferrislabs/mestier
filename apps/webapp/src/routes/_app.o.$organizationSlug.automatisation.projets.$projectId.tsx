import { createFileRoute } from '@tanstack/react-router'
import { AutomationWorkflowsFeature } from '#/pages/automation/feature/automation-workflows-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/projets/$projectId',
)({
	component: AutomationProjectPage,
})

function AutomationProjectPage() {
	const { projectId } = Route.useParams()

	return <AutomationWorkflowsFeature folderId={projectId} />
}
