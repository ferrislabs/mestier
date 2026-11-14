import { createFileRoute } from '@tanstack/react-router'
import { SurveyResponsesFeature } from '#/pages/survey/feature/survey-responses-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/$surveyId/reponses',
)({
	component: SurveyResponsesPage,
})

function SurveyResponsesPage() {
	const { surveyId } = Route.useParams()

	return <SurveyResponsesFeature surveyId={surveyId} />
}
