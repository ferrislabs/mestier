import { createFileRoute } from '@tanstack/react-router'
import { ExtraFeature, type ExtraTab } from '#/pages/hr/feature/extra-feature'
import { isAbsenceStatus } from '#/pages/hr/lib/rh/absences-view'
import type { ExtraStatus } from '#/pages/hr/lib/rh/types'

interface ExtraSearch {
	onglet?: ExtraTab
	statut?: ExtraStatus
	membre?: string
}

function validateSearch(raw: Record<string, unknown>): ExtraSearch {
	if (raw.onglet === 'majorations') return { onglet: 'majorations' }
	return {
		...(isAbsenceStatus(raw.statut) ? { statut: raw.statut } : {}),
		...(typeof raw.membre === 'string' && raw.membre !== ''
			? { membre: raw.membre }
			: {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/rh/extra/')({
	validateSearch,
	component: ExtraRoute,
})

function ExtraRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ExtraFeature
			tab={search.onglet ?? 'declarations'}
			onTabChange={(tab) =>
				navigate({
					replace: true,
					search: tab === 'majorations' ? { onglet: 'majorations' } : {},
				})
			}
			status={search.statut ?? 'all'}
			memberId={search.membre ?? 'all'}
			onFiltersChange={(patch) =>
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
					to: '/o/$organizationSlug/rh/extra/nouvelle',
					search: {
						...(initial?.memberId ? { membre: initial.memberId } : {}),
						...(initial?.date ? { date: initial.date } : {}),
					},
				})
			}}
		/>
	)
}
