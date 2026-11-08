import { isActiveAbsence } from '#/pages/hr/lib/rh/absences'
import { isActiveExtra } from '#/pages/hr/lib/rh/extra'
import type { RhState } from '#/pages/hr/lib/rh/types'

export interface SidebarCounts {
	active: number
	requests: number
	pending: number
	mine: number
	types: number
	extraRequests: number
	extraPending: number
	extraMine: number
	surcharges: number
}

export function sidebarCounts(
	state: RhState,
	currentMemberId: string,
): SidebarCounts {
	return {
		active: state.members.filter((member) => member.status !== 'archived')
			.length,
		requests: state.absences.length,
		pending: state.absences.filter((absence) => absence.status === 'pending')
			.length,
		mine: state.absences.filter(
			(absence) =>
				absence.memberId === currentMemberId && isActiveAbsence(absence),
		).length,
		types: state.absenceTypes.filter((type) => !type.archived).length,
		extraRequests: state.extras.length,
		extraPending: state.extras.filter((extra) => extra.status === 'pending')
			.length,
		extraMine: state.extras.filter(
			(extra) => extra.memberId === currentMemberId && isActiveExtra(extra),
		).length,
		surcharges: state.surcharges.filter((surcharge) => !surcharge.archived)
			.length,
	}
}
