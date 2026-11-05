import { daysInRange, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { validateWorkDay } from '#/pages/hr/lib/rh/schedule'
import type {
	ActionContext,
	DayOverride,
	DayOverrideKind,
	IsoDate,
	LunchBreak,
	RhState,
	Weekday,
} from '#/pages/hr/lib/rh/types'

export const WORKING_WEEKDAYS: readonly Weekday[] = [0, 1, 2, 3, 4]

export type DayPlan = {
	memberId: string
	startDate: IsoDate
	endDate: IsoDate
	weekdays: readonly Weekday[]
	kind: DayOverrideKind
	start: string
	end: string
	lunch: LunchBreak | null
	note: string
}

export function planDates(
	plan: Pick<DayPlan, 'startDate' | 'endDate' | 'weekdays'>,
) {
	return daysInRange(plan.startDate, plan.endDate).filter((date) =>
		plan.weekdays.includes(weekdayOf(date)),
	)
}

export function validatePlan(state: RhState, plan: DayPlan): string[] {
	if (!state.members.some((member) => member.id === plan.memberId))
		return ['Choisis un membre.']
	if (!plan.startDate || !plan.endDate)
		return ['Renseigne la période à planifier.']
	if (plan.endDate < plan.startDate)
		return ['La fin de la période précède son début.']
	if (plan.weekdays.length === 0)
		return ['Choisis au moins un jour de la semaine.']
	if (planDates(plan).length === 0)
		return ['Aucun jour de la période ne correspond aux jours choisis.']
	if (plan.kind === 'off') return []
	const error = validateWorkDay({
		start: plan.start,
		end: plan.end,
		lunch: plan.lunch,
	})
	return error ? [error] : []
}

export function planDays(
	state: RhState,
	_context: ActionContext,
	plan: DayPlan,
): RhState {
	if (validatePlan(state, plan).length > 0) return state
	const dates = new Set(planDates(plan))
	const created: DayOverride[] = [...dates].map((date) => ({
		memberId: plan.memberId,
		date,
		kind: plan.kind,
		start: plan.kind === 'work' ? plan.start : '',
		end: plan.kind === 'work' ? plan.end : '',
		lunch: plan.kind === 'work' ? plan.lunch : null,
		note: plan.note.trim(),
	}))
	return {
		...state,
		overrides: [
			...state.overrides.filter(
				(override) =>
					override.memberId !== plan.memberId || !dates.has(override.date),
			),
			...created,
		],
	}
}

export function clearPlanning(
	state: RhState,
	memberId: string,
	startDate: IsoDate,
	endDate: IsoDate,
): RhState {
	return {
		...state,
		overrides: state.overrides.filter(
			(override) =>
				override.memberId !== memberId ||
				override.date < startDate ||
				override.date > endDate,
		),
	}
}
