import {
	DEADLINE_LABELS,
	type DeadlineKind,
	findMemberById,
	upcomingDeadlines,
} from '#/pages/hr/lib/rh/members'
import type { IsoDate, RhState } from '#/pages/hr/lib/rh/types'

export interface SidebarDeadline {
	key: string
	memberId: string
	kind: DeadlineKind
	label: string
	note: string
	title: string
}

const SHORT: Record<DeadlineKind, string> = {
	trial: 'essai',
	contract_end: 'contrat',
	work_permit: 'titre',
}

export function sidebarDeadlines(
	state: RhState,
	today: IsoDate,
	limit: number,
): SidebarDeadline[] {
	return upcomingDeadlines(state, today)
		.slice(0, limit)
		.flatMap((deadline) => {
			const member = findMemberById(state, deadline.memberId)
			return member
				? [
						{
							key: `${deadline.memberId}-${deadline.kind}`,
							memberId: member.id,
							kind: deadline.kind,
							label: `${member.firstName} · ${SHORT[deadline.kind]}`,
							note: `${deadline.daysLeft} j`,
							title: `${DEADLINE_LABELS[deadline.kind]} de ${member.firstName} ${member.lastName}, dans ${deadline.daysLeft} jours`,
						},
					]
				: []
		})
}
