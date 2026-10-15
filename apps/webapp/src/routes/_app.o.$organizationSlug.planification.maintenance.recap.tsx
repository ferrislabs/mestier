import { createFileRoute } from '@tanstack/react-router'
import { RecapFeature } from '#/pages/maintenance/feature/recap-feature'
import { recapSearchSchema } from '#/pages/maintenance/lib/recap-week'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/maintenance/recap',
)({
	validateSearch: (search) => recapSearchSchema.parse(search),
	component: RecapPage,
})

function RecapPage() {
	const { week } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<RecapFeature
			week={week}
			onWeekChange={(next) =>
				void navigate({ search: { week: next }, replace: true })
			}
		/>
	)
}
