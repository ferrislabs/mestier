import { createFileRoute } from '@tanstack/react-router'
import { TrainingsPageFeature } from '#/pages/cursus/catalog/feature/trainings-page-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/formations/',
)({
	component: TrainingsPageFeature,
})
