import { createFileRoute } from '@tanstack/react-router'
import { DoeModelPageFeature } from '#/pages/doe-models/feature/doe-model-page-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/doe-models_/$modelId',
)({
	component: DoeModelPage,
})

function DoeModelPage() {
	const { modelId } = Route.useParams()

	return <DoeModelPageFeature modelId={modelId} />
}
