import { createFileRoute } from '@tanstack/react-router'
import { ContractsFeature } from '#/pages/maintenance/feature/contracts-feature'
import {
	resolveSeason,
	seasonSearchSchema,
} from '#/pages/maintenance/lib/season-search'
import { todayIsoDate } from '#/pages/planning/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/maintenance/contracts',
)({
	validateSearch: (search) => seasonSearchSchema.parse(search),
	component: MaintenanceContractsPage,
})

function MaintenanceContractsPage() {
	const { season } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ContractsFeature
			season={resolveSeason(season, todayIsoDate())}
			onSeasonChange={(next) => navigate({ search: { season: next } })}
		/>
	)
}
