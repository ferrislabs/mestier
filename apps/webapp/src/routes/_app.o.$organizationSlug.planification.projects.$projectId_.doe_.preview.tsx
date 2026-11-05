import { createFileRoute } from '@tanstack/react-router'
import { DoePreviewFeature } from '#/pages/doe/feature/doe-preview-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/projects/$projectId_/doe_/preview',
)({
	component: DoePreviewPage,
})

function DoePreviewPage() {
	const { projectId } = Route.useParams()

	return <DoePreviewFeature projectId={projectId} />
}
