import { createFileRoute } from '@tanstack/react-router'
import {
	AbsencesFeature,
	type AbsencesTab,
} from '#/pages/hr/feature/absences-feature'
import { isAbsenceStatus } from '#/pages/hr/lib/rh/absences-view'
import type { AbsenceStatus } from '#/pages/hr/lib/rh/types'

interface AbsencesSearch {
	onglet?: AbsencesTab
	statut?: AbsenceStatus
	membre?: string
}

function validateSearch(raw: Record<string, unknown>): AbsencesSearch {
	if (raw.onglet === 'motifs') return { onglet: 'motifs' }
	return {
		...(isAbsenceStatus(raw.statut) ? { statut: raw.statut } : {}),
		...(typeof raw.membre === 'string' && raw.membre !== ''
			? { membre: raw.membre }
			: {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/rh/absences/')({
	validateSearch,
	component: AbsencesRoute,
})

function AbsencesRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<AbsencesFeature
			tab={search.onglet ?? 'demandes'}
			onTabChange={(tab) =>
				navigate({
					replace: true,
					search: tab === 'motifs' ? { onglet: 'motifs' } : {},
				})
			}
			historyStatus={search.statut ?? 'all'}
			historyMemberId={search.membre ?? 'all'}
			onHistoryFiltersChange={(patch) =>
				navigate({
					replace: true,
					search: {
						...(patch.status === 'all' ? {} : { statut: patch.status }),
						...(patch.memberId === 'all' ? {} : { membre: patch.memberId }),
					},
				})
			}
			creating={false}
			onCreatingChange={(open, initial) => {
				if (!open) return
				navigate({
					to: '/o/$organizationSlug/rh/absences/nouvelle',
					search: {
						...(initial?.memberId ? { membre: initial.memberId } : {}),
						...(initial?.startDate ? { date: initial.startDate } : {}),
					},
				})
			}}
		/>
	)
}
