import { endOfMonth, startOfMonth, weekdayOf } from '#/pages/hr/lib/rh/dates'
import {
	type DayPlan,
	planDates,
	WORKING_WEEKDAYS,
} from '#/pages/hr/lib/rh/day-planning'
import type {
	DayOverrideKind,
	IsoDate,
	RhState,
	Weekday,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export type PlanPeriod = { startDate: IsoDate; endDate: IsoDate }

export function defaultPlanPeriod(month: IsoDate, today: IsoDate): PlanPeriod {
	const first = startOfMonth(month)
	const last = endOfMonth(month)
	return {
		startDate: today >= first && today <= last ? today : first,
		endDate: last,
	}
}

export function emptyPlan(
	memberId: string,
	period: PlanPeriod,
	day: WorkDay,
): DayPlan {
	return {
		memberId,
		startDate: period.startDate,
		endDate: period.endDate,
		weekdays: WORKING_WEEKDAYS,
		kind: 'work',
		start: day.start,
		end: day.end,
		lunch: day.lunch ? { ...day.lunch } : null,
		note: '',
	}
}

export function dayPlanOf(
	memberId: string,
	date: IsoDate,
	kind: DayOverrideKind,
	day: WorkDay,
): DayPlan {
	return {
		...emptyPlan(memberId, { startDate: date, endDate: date }, day),
		weekdays: [weekdayOf(date)],
		kind,
	}
}

export function toWeekdays(values: readonly string[]): Weekday[] {
	return values.map((value) => Number(value) as Weekday)
}

export function fromWeekdays(weekdays: readonly Weekday[]): string[] {
	return weekdays.map((weekday) => String(weekday))
}

export function countPlanned(
	state: RhState,
	memberId: string,
	startDate: IsoDate,
	endDate: IsoDate,
): number {
	return state.overrides.filter(
		(override) =>
			override.memberId === memberId &&
			override.date >= startDate &&
			override.date <= endDate,
	).length
}

export function planSummary(state: RhState, plan: DayPlan): string {
	const dates = planDates(plan)
	if (dates.length === 0) return 'Aucun jour à planifier.'
	const replaced = dates.filter((date) =>
		state.overrides.some(
			(override) =>
				override.memberId === plan.memberId && override.date === date,
		),
	).length
	const base =
		dates.length === 1
			? '1 jour sera planifié'
			: `${dates.length} jours seront planifiés`
	return replaced > 0
		? `${base}, dont ${replaced} déjà planifié${replaced > 1 ? 's' : ''} (${replaced > 1 ? 'ils seront remplacés' : 'il sera remplacé'}).`
		: `${base}.`
}
