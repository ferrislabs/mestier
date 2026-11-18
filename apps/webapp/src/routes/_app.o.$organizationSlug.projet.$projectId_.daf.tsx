import { createFileRoute } from '@tanstack/react-router'
import { DafFeature } from '#/pages/daf/feature/daf-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/$projectId_/daf',
)({
	component: DafPage,
})

function DafPage() {
	const { projectId } = Route.useParams()
	return <DafFeature projectId={projectId} />
}
