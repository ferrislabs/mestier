import { createFileRoute } from '@tanstack/react-router'
import {
	PlanningFeature,
	type PlanningView,
} from '#/pages/hr/feature/planning-feature'
import { isIsoDate, todayIso } from '#/pages/hr/lib/rh/dates'
import type { IsoDate } from '#/pages/hr/lib/rh/types'

interface PlanningSearch {
	view?: PlanningView
	date?: IsoDate
}

function validateSearch(raw: Record<string, unknown>): PlanningSearch {
	return {
		...(raw.view === 'month' || raw.view === 'week' ? { view: raw.view } : {}),
		...(isIsoDate(raw.date) ? { date: raw.date } : {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/planning')({
	validateSearch,
	component: PlanningRoute,
})

function PlanningRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<PlanningFeature
			view={search.view ?? 'week'}
			date={search.date ?? todayIso()}
			onChange={(patch) =>
				navigate({
					replace: true,
					search: (previous) =>
						validateSearch({
							...previous,
							...(patch.view ? { view: patch.view } : {}),
							...(patch.date ? { date: patch.date } : {}),
						}),
				})
			}
		/>
	)
}
