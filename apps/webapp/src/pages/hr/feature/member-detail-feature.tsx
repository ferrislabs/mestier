import { useUpdateMember } from '#/hooks/use-reference-catalog'
import { useMemberOverrides } from '#/pages/hr/feature/use-override-period'
import { monthWeeks } from '#/pages/hr/lib/rh/calendar-model'
import { startOfMonth, todayIso } from '#/pages/hr/lib/rh/dates'
import { findMemberById } from '#/pages/hr/lib/rh/members'
import type { MemberTab } from '#/pages/hr/lib/rh/rubriques'
import type { IsoDate } from '#/pages/hr/lib/rh/types'
import { MemberDetailPage } from '#/pages/hr/ui/rh/members/member-detail-page'
import {
	useRhOrganizationId,
	useRhState,
} from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'

export interface MemberDetailFeatureProps {
	memberId: string
	tab: MemberTab
	month: IsoDate | null
	onChange: (patch: { month: IsoDate | null }) => void
}

export function MemberDetailFeature(props: MemberDetailFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement de la fiche…">
			<MemberDetailScreen {...props} />
		</RhScreen>
	)
}

function MemberDetailScreen({
	memberId,
	tab,
	month,
	onChange,
}: MemberDetailFeatureProps) {
	const state = useRhState()
	const updateMember = useUpdateMember()
	const today = todayIso()
	const organizationId = useRhOrganizationId()
	const shownMonth = month ?? startOfMonth(today)
	const weeks = monthWeeks(shownMonth)
	useMemberOverrides(
		organizationId,
		memberId,
		weeks[0][0],
		weeks[weeks.length - 1][6],
		tab === 'emploi-du-temps',
	)

	return (
		<MemberDetailPage
			state={state}
			member={findMemberById(state, memberId)}
			today={today}
			tab={tab}
			month={month ?? startOfMonth(today)}
			onMonthChange={(next) => onChange({ month: next })}
			onRename={({ firstName, lastName }) =>
				updateMember.mutate({
					path: { member_id: memberId },
					body: {
						first_name: firstName.trim() || null,
						last_name: lastName.trim(),
					},
				})
			}
		/>
	)
}
