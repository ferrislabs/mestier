import { createFileRoute } from '@tanstack/react-router'
import { AutomationCredentialsFeature } from '#/pages/automation/feature/automation-credentials-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/credentials',
)({
	component: AutomationCredentialsFeature,
})
