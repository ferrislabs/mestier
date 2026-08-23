import { createFileRoute } from '@tanstack/react-router'
import { ProjectTemplatesFeature } from '#/pages/project-templates/feature/project-templates-feature'
import { projectTemplatesSearchSchema } from '#/pages/project-templates/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/project-templates',
)({
	validateSearch: (search) => projectTemplatesSearchSchema.parse(search),
	component: ProjectTemplatesPage,
})

function ProjectTemplatesPage() {
	const { archived } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ProjectTemplatesFeature
			includeArchived={archived}
			onIncludeArchivedChange={(archived) =>
				navigate({ search: (prev) => ({ ...prev, archived }) })
			}
		/>
	)
}
