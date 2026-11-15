import { createFileRoute } from '@tanstack/react-router'
import { SurveyNoSelection } from '#/pages/survey/ui/survey-no-selection'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/',
)({
	component: SurveyNoSelection,
})
