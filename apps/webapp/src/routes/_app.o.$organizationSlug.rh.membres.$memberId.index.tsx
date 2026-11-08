import { createFileRoute } from '@tanstack/react-router'
import { MemberDetailFeature } from '#/pages/hr/feature/member-detail-feature'
import { isIsoDate } from '#/pages/hr/lib/rh/dates'
import {
	DEFAULT_TAB,
	isMemberTab,
	type MemberTab,
} from '#/pages/hr/lib/rh/rubriques'
import type { IsoDate } from '#/pages/hr/lib/rh/types'

interface MemberSearch {
	onglet?: MemberTab
	mois?: IsoDate
}

function validateSearch(raw: Record<string, unknown>): MemberSearch {
	return {
		...(isMemberTab(raw.onglet) ? { onglet: raw.onglet } : {}),
		...(isIsoDate(raw.mois) ? { mois: raw.mois } : {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/rh/membres/$memberId/',
)({
	validateSearch,
	component: MemberRoute,
})

function MemberRoute() {
	const { memberId } = Route.useParams()
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<MemberDetailFeature
			memberId={memberId}
			tab={search.onglet ?? DEFAULT_TAB}
			month={search.mois ?? null}
			onChange={(patch) =>
				navigate({
					replace: true,
					search: (previous) => ({
						...previous,
						mois: patch.month ?? undefined,
					}),
				})
			}
		/>
	)
}
