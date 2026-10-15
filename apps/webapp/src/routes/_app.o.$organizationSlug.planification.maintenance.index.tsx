import { createFileRoute } from '@tanstack/react-router'
import { GridFeature } from '#/pages/maintenance/feature/grid-feature'
import {
	resolveSeason,
	seasonSearchSchema,
} from '#/pages/maintenance/lib/season-search'
import { todayIsoDate } from '#/pages/planning/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/maintenance/',
)({
	validateSearch: (search) => seasonSearchSchema.parse(search),
	component: MaintenanceGridPage,
})

function MaintenanceGridPage() {
	const { season } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<GridFeature
			season={resolveSeason(season, todayIsoDate())}
			onSeasonChange={(next) => navigate({ search: { season: next } })}
		/>
	)
}
