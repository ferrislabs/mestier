import { createFileRoute } from '@tanstack/react-router'
import { AbsencesFeature } from '#/pages/hr/feature/absences-feature'
import { isIsoDate } from '#/pages/hr/lib/rh/dates'

interface NewAbsenceSearch {
	membre?: string
	date?: string
}

function validateSearch(raw: Record<string, unknown>): NewAbsenceSearch {
	return {
		...(typeof raw.membre === 'string' && raw.membre !== ''
			? { membre: raw.membre }
			: {}),
		...(isIsoDate(raw.date) ? { date: raw.date } : {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/rh/absences/nouvelle',
)({
	validateSearch,
	component: NewAbsenceRoute,
})

function NewAbsenceRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<AbsencesFeature
			historyStatus="all"
			historyMemberId="all"
			onHistoryFiltersChange={(patch) =>
				navigate({
					to: '/o/$organizationSlug/rh/absences',
					search: {
						...(patch.status === 'all' ? {} : { statut: patch.status }),
						...(patch.memberId === 'all' ? {} : { membre: patch.memberId }),
					},
				})
			}
			tab="demandes"
			onTabChange={(tab) =>
				navigate({
					to: '/o/$organizationSlug/rh/absences',
					search: tab === 'motifs' ? { onglet: 'motifs' } : {},
				})
			}
			creating
			initial={{ memberId: search.membre, startDate: search.date }}
			onCreatingChange={(open) => {
				if (!open)
					navigate({ to: '/o/$organizationSlug/rh/absences', search: {} })
			}}
		/>
	)
}
