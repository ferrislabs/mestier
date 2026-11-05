import { createFileRoute } from '@tanstack/react-router'
import { ProjectsFeature } from '#/pages/projects/feature/projects-feature'
import {
	filtersOfSearch,
	searchOfFilters,
} from '#/pages/projects/lib/list-search'
import { projectsSearchSchema } from '#/pages/projects/types'

export const Route = createFileRoute('/_app/o/$organizationSlug/projet/')({
	validateSearch: (search) => projectsSearchSchema.parse(search),
	component: ProjectsPage,
})

function ProjectsPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ProjectsFeature
			filters={filtersOfSearch(search)}
			highlightedProjectId={search.projectId}
			onFiltersChange={(filters) =>
				navigate({
					replace: true,
					search: searchOfFilters(filters, search.projectId),
				})
			}
		/>
	)
}
