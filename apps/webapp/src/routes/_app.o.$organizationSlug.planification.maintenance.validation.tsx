import { createFileRoute } from '@tanstack/react-router'
import { ValidationFeature } from '#/pages/maintenance/feature/validation-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/maintenance/validation',
)({
	component: ValidationFeature,
})
