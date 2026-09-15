import { createFileRoute } from '@tanstack/react-router'
import { AutomationWorkflowsFeature } from '#/pages/automation/feature/automation-workflows-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/',
)({
	component: AutomationWorkflowsFeature,
})
