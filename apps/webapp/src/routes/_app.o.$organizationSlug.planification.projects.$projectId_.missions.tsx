import { createFileRoute } from '@tanstack/react-router'
import { MissionsFeature } from '#/pages/projects/feature/missions-feature'
import {
	parseMissionSearch,
	searchOfMissionFilter,
} from '#/pages/projects/lib/mission-list'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/projects/$projectId_/missions',
)({
	validateSearch: parseMissionSearch,
	component: MissionsPage,
})

function MissionsPage() {
	const { projectId } = Route.useParams()
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<MissionsFeature
			projectId={projectId}
			filter={search.statut ?? 'toutes'}
			onFilterChange={(filter) =>
				navigate({ replace: true, search: searchOfMissionFilter(filter) })
			}
		/>
	)
}
