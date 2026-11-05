import { createFileRoute } from '@tanstack/react-router'
import { ExtraFeature, type ExtraTab } from '#/pages/hr/feature/extra-feature'
import { isAbsenceStatus } from '#/pages/hr/lib/rh/absences-view'
import type { ExtraStatus } from '#/pages/hr/lib/rh/types'

interface ExtraSearch {
	tab?: ExtraTab
	status?: ExtraStatus
	member?: string
}

function validateSearch(raw: Record<string, unknown>): ExtraSearch {
	if (raw.tab === 'majorations') return { tab: 'majorations' }
	return {
		...(isAbsenceStatus(raw.status) ? { status: raw.status } : {}),
		...(typeof raw.member === 'string' && raw.member !== ''
			? { member: raw.member }
			: {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/extra/')({
	validateSearch,
	component: ExtraRoute,
})

function ExtraRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ExtraFeature
			tab={search.tab ?? 'declarations'}
			onTabChange={(tab) =>
				navigate({
					replace: true,
					search: tab === 'majorations' ? { tab: 'majorations' } : {},
				})
			}
			status={search.status ?? 'all'}
			memberId={search.member ?? 'all'}
			onFiltersChange={(patch) =>
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
					to: '/o/$organizationSlug/hr/extra/new',
					search: {
						...(initial?.memberId ? { member: initial.memberId } : {}),
						...(initial?.date ? { date: initial.date } : {}),
					},
				})
			}}
		/>
	)
}
