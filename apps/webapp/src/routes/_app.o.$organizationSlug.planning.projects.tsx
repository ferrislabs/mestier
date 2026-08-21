import { createFileRoute } from '@tanstack/react-router'
import { ProjectsFeature } from '#/pages/projects/feature/projects-feature'
import { projectsSearchSchema } from '#/pages/projects/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/projects',
)({
	validateSearch: (search) => projectsSearchSchema.parse(search),
	component: ProjectsPage,
})

function ProjectsPage() {
	const { projectId, archived } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ProjectsFeature
			highlightedProjectId={projectId}
			includeArchived={archived}
			onIncludeArchivedChange={(archived) =>
				navigate({ search: (prev) => ({ ...prev, archived }) })
			}
		/>
	)
}
