import { formatDuration } from '#/pages/hr/lib/rh/extra'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'

export const RECENT_EXTRAS_LIMIT = 5

export function recentExtras(
	state: RhState,
	memberId: string,
	limit = RECENT_EXTRAS_LIMIT,
): ExtraPeriod[] {
	return state.extras
		.filter((extra) => extra.memberId === memberId)
		.sort(
			(a, b) => b.date.localeCompare(a.date) || b.start.localeCompare(a.start),
		)
		.slice(0, limit)
}

export function totalLabel(minutes: number): string {
	return minutes === 0 ? '0 h' : formatDuration(minutes)
}
