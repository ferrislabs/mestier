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
	tab?: MemberTab
	month?: IsoDate
}

function validateSearch(raw: Record<string, unknown>): MemberSearch {
	return {
		...(isMemberTab(raw.tab) ? { tab: raw.tab } : {}),
		...(isIsoDate(raw.month) ? { month: raw.month } : {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/hr/team/$memberId/',
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
			tab={search.tab ?? DEFAULT_TAB}
			month={search.month ?? null}
			onChange={(patch) =>
				navigate({
					replace: true,
					search: (previous) => ({
						...previous,
						month: patch.month ?? undefined,
					}),
				})
			}
		/>
	)
}
