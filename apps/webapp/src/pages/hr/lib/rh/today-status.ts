import { resolveDay } from '#/pages/hr/lib/rh/planning'
import type {
	DayPart,
	IsoDate,
	LunchBreak,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type TodayStatus =
	| { kind: 'inactive' }
	| { kind: 'work'; start: string; end: string; lunch: LunchBreak | null }
	| { kind: 'off'; label?: string }
	| { kind: 'unplanned' }
	| { kind: 'holiday'; name: string }
	| {
			kind: 'absence'
			typeId: string
			part: 'full' | DayPart
			pending: boolean
	  }

export function todayStatus(
	state: RhState,
	member: Member,
	today: IsoDate,
): TodayStatus {
	if (member.status === 'archived') return { kind: 'inactive' }
	const day = resolveDay(state, member, today)
	const { morning, afternoon } = day
	if (morning.kind === 'absence' || afternoon.kind === 'absence') {
		const halves = [morning, afternoon].flatMap((half) =>
			half.kind === 'absence' ? [half] : [],
		)
		const first = halves[0]
		return {
			kind: 'absence',
			typeId: first.typeId,
			part:
				halves.length === 2
					? 'full'
					: morning.kind === 'absence'
						? 'morning'
						: 'afternoon',
			pending: halves.every((half) => half.pending),
		}
	}
	if (morning.kind === 'inactive' && afternoon.kind === 'inactive')
		return { kind: 'inactive' }
	if (morning.kind === 'holiday') return { kind: 'holiday', name: morning.name }
	if (morning.kind === 'work')
		return {
			kind: 'work',
			start: morning.start,
			end: morning.end,
			lunch: morning.lunch,
		}
	if (morning.kind === 'unplanned') return { kind: 'unplanned' }
	return morning.kind === 'off' && morning.label
		? { kind: 'off', label: morning.label }
		: { kind: 'off' }
}
