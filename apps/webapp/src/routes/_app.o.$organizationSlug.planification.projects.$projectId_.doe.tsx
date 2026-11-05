import { createFileRoute } from '@tanstack/react-router'
import { DoeFeature } from '#/pages/doe/feature/doe-feature'
import { tabSearch, validateDoeSearch } from '#/pages/doe/lib/rubriques'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/projects/$projectId_/doe',
)({
	validateSearch: validateDoeSearch,
	component: DoePage,
})

function DoePage() {
	const { projectId } = Route.useParams()
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<DoeFeature
			projectId={projectId}
			tab={search.onglet}
			onTabChange={(tab) => navigate({ replace: true, search: tabSearch(tab) })}
		/>
	)
}
