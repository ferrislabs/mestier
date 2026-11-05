import { createFileRoute } from '@tanstack/react-router'
import {
	AbsencesFeature,
	type AbsencesTab,
} from '#/pages/hr/feature/absences-feature'
import { isAbsenceStatus } from '#/pages/hr/lib/rh/absences-view'
import type { AbsenceStatus } from '#/pages/hr/lib/rh/types'

interface AbsencesSearch {
	tab?: AbsencesTab
	status?: AbsenceStatus
	member?: string
}

function validateSearch(raw: Record<string, unknown>): AbsencesSearch {
	if (raw.tab === 'motifs') return { tab: 'motifs' }
	return {
		...(isAbsenceStatus(raw.status) ? { status: raw.status } : {}),
		...(typeof raw.member === 'string' && raw.member !== ''
			? { member: raw.member }
			: {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/absences/')({
	validateSearch,
	component: AbsencesRoute,
})

function AbsencesRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<AbsencesFeature
			tab={search.tab ?? 'demandes'}
			onTabChange={(tab) =>
				navigate({
					replace: true,
					search: tab === 'motifs' ? { tab: 'motifs' } : {},
				})
			}
			historyStatus={search.status ?? 'all'}
			historyMemberId={search.member ?? 'all'}
			onHistoryFiltersChange={(patch) =>
				navigate({
					replace: true,
					search: {
						...(patch.status === 'all' ? {} : { status: patch.status }),
						...(patch.memberId === 'all' ? {} : { member: patch.memberId }),
					},
				})
			}
			creating={false}
			onCreatingChange={(open, initial) => {
				if (!open) return
				navigate({
					to: '/o/$organizationSlug/hr/absences/new',
					search: {
						...(initial?.memberId ? { member: initial.memberId } : {}),
						...(initial?.startDate ? { date: initial.startDate } : {}),
					},
				})
			}}
		/>
	)
}
