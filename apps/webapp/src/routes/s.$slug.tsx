import { createFileRoute } from '@tanstack/react-router'
import { PublicSurveyFeature } from '#/pages/survey-respond/feature/public-survey-feature'

export const Route = createFileRoute('/s/$slug')({
	component: PublicSurveyPage,
})

function PublicSurveyPage() {
	const { slug } = Route.useParams()
	return <PublicSurveyFeature key={slug} slug={slug} />
}
