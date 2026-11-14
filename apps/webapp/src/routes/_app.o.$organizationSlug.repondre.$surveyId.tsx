import { createFileRoute } from '@tanstack/react-router'
import { MemberSurveyFeature } from '#/pages/survey-respond/feature/member-survey-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/repondre/$surveyId',
)({
	component: MemberSurveyPage,
})

function MemberSurveyPage() {
	const { surveyId } = Route.useParams()
	return <MemberSurveyFeature key={surveyId} surveyId={surveyId} />
}
