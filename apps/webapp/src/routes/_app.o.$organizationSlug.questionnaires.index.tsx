import { createFileRoute } from '@tanstack/react-router'
import { SurveysListFeature } from '#/pages/survey/feature/surveys-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/',
)({
	component: SurveysListFeature,
})
