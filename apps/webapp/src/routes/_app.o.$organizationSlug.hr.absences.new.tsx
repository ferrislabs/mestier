import { createFileRoute } from '@tanstack/react-router'
import { AbsencesFeature } from '#/pages/hr/feature/absences-feature'
import { isIsoDate } from '#/pages/hr/lib/rh/dates'

interface NewAbsenceSearch {
	member?: string
	date?: string
}

function validateSearch(raw: Record<string, unknown>): NewAbsenceSearch {
	return {
		...(typeof raw.member === 'string' && raw.member !== ''
			? { member: raw.member }
			: {}),
		...(isIsoDate(raw.date) ? { date: raw.date } : {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/hr/absences/new',
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
					to: '/o/$organizationSlug/hr/absences',
					search: {
						...(patch.status === 'all' ? {} : { status: patch.status }),
						...(patch.memberId === 'all' ? {} : { member: patch.memberId }),
					},
				})
			}
			tab="demandes"
			onTabChange={(tab) =>
				navigate({
					to: '/o/$organizationSlug/hr/absences',
					search: tab === 'motifs' ? { tab: 'motifs' } : {},
				})
			}
			creating
			initial={{ memberId: search.member, startDate: search.date }}
			onCreatingChange={(open) => {
				if (!open)
					navigate({ to: '/o/$organizationSlug/hr/absences', search: {} })
			}}
		/>
	)
}
