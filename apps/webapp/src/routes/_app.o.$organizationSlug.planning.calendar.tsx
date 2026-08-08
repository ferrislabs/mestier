import { createFileRoute } from '@tanstack/react-router'
import { PlanningCalendarFeature } from '#/pages/planning/feature/planning-calendar-feature'
import { planningSearchSchema } from '#/pages/planning/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/calendar',
)({
	validateSearch: (search) => planningSearchSchema.parse(search),
	component: PlanningCalendarPage,
})

function PlanningCalendarPage() {
	const { view, date } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<PlanningCalendarFeature
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
