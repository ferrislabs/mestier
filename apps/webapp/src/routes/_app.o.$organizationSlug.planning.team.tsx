import { createFileRoute } from '@tanstack/react-router'
import { PlanningTeamFeature } from '#/pages/planning/feature/planning-team-feature'
import { planningSearchSchema } from '#/pages/planning/types'

export const Route = createFileRoute('/_app/o/$organizationSlug/planning/team')(
	{
		// The view lives in the URL — `?view=week&date=2026-08-07` — so the
		// planning screen is shareable, survives a reload, and the back button
		// behaves (see the planning design doc's "état de vue dans l'URL"
		// section). Every field falls back rather than throwing, so a malformed
		// query string still lands somewhere sensible.
		validateSearch: (search) => planningSearchSchema.parse(search),
		component: PlanningTeamPage,
	},
)

function PlanningTeamPage() {
	const { view, date } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<PlanningTeamFeature
			view={view}
			date={date}
			onViewChange={(view) =>
				navigate({ search: (prev) => ({ ...prev, view }) })
			}
			onDateChange={(date) =>
				navigate({ search: (prev) => ({ ...prev, date }) })
			}
		/>
	)
}
