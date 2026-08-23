import { createFileRoute } from '@tanstack/react-router'
import { ProjectDetailFeature } from '#/pages/projects/feature/project-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/projects/$projectId',
)({
	component: ProjectDetailPage,
})

function ProjectDetailPage() {
	const { projectId } = Route.useParams()

	return <ProjectDetailFeature projectId={projectId} />
}
