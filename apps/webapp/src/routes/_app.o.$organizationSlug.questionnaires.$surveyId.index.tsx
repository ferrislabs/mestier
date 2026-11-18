import { createFileRoute } from '@tanstack/react-router'
import { SurveyEditorFeature } from '#/pages/survey/feature/survey-editor-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/$surveyId/',
)({
	component: SurveyEditorPage,
})

function SurveyEditorPage() {
	const { surveyId } = Route.useParams()

	return <SurveyEditorFeature surveyId={surveyId} />
}
