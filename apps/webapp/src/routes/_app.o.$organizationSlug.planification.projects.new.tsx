import { createFileRoute } from '@tanstack/react-router'
import { ProjectCreateFeature } from '#/pages/projects/feature/project-create-feature'
import { parseNewSearch } from '#/pages/projects/lib/project-form'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/projects/new',
)({
	validateSearch: parseNewSearch,
	component: NewProjectPage,
})

function NewProjectPage() {
	const search = Route.useSearch()

	return <ProjectCreateFeature customerId={search.client} />
}
