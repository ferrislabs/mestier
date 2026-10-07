import { createFileRoute } from '@tanstack/react-router'
import { DoeFeature } from '#/pages/doe/feature/doe-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/projects/$projectId_/doe',
)({
	component: DoePage,
})

function DoePage() {
	const { projectId } = Route.useParams()

	return <DoeFeature projectId={projectId} />
}
