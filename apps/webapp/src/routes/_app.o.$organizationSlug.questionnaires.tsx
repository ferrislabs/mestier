import { createFileRoute } from '@tanstack/react-router'
import { SurveysLayoutFeature } from '#/pages/survey/feature/surveys-layout-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires',
)({
	component: SurveysLayoutFeature,
})
