import { createFileRoute } from '@tanstack/react-router'
import { DoesFeature } from '#/pages/does/feature/does-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/does',
)({
	component: DoesFeature,
})
