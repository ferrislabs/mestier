import { createFileRoute } from '@tanstack/react-router'
import { SupportIntegrationFeature } from '#/pages/support/feature/support-integration-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/support/integration',
)({
	component: SupportIntegrationFeature,
})
