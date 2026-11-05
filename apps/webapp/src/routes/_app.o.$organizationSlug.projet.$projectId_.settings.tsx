import { createFileRoute } from '@tanstack/react-router'
import { ProjectSettingsFeature } from '#/pages/projects/feature/project-settings-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/$projectId_/settings',
)({
	component: ProjectSettingsPage,
})

function ProjectSettingsPage() {
	const { projectId } = Route.useParams()

	return <ProjectSettingsFeature projectId={projectId} />
}
