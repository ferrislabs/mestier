import {
	countsOn,
	coversPart,
	findType,
	isActiveAbsence,
} from '#/pages/hr/lib/rh/absences'
import {
	blockLabelOn,
	contractOn,
	isVariableOn,
	scheduleOn,
} from '#/pages/hr/lib/rh/contracts'
import { weekdayOf } from '#/pages/hr/lib/rh/dates'
import { holidayName } from '#/pages/hr/lib/rh/holidays'
import type {
	ActionContext,
	DayOverride,
	DayPart,
	IsoDate,
	LunchBreak,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type HalfDay =
	| {
			kind: 'work'
			start: string
			end: string
			lunch: LunchBreak | null
			overridden: boolean
	  }
	| { kind: 'off'; overridden: boolean; label?: string }
	| { kind: 'unplanned' }
	| { kind: 'holiday'; name: string }
	| { kind: 'absence'; absenceId: string; typeId: string; pending: boolean }
	| { kind: 'inactive' }

export type DayStatus = {
	variable: boolean
	date: IsoDate
	morning: HalfDay
	afternoon: HalfDay
	override: DayOverride | null
}

export function isEmployedOn(member: Member, date: IsoDate): boolean {
	if (contractOn(member, date) === null) return false
	return !(member.archivedAt && date > member.archivedAt)
}

export function findOverride(
	state: RhState,
	memberId: string,
	date: IsoDate,
): DayOverride | null {
	return (
		state.overrides.find(
			(override) => override.memberId === memberId && override.date === date,
		) ?? null
	)
}

function baseHalf(
	member: Member,
	date: IsoDate,
	override: DayOverride | null,
	variable: boolean,
): HalfDay {
	if (override?.kind === 'work')
		return {
			kind: 'work',
			start: override.start,
			end: override.end,
			lunch: override.lunch ?? null,
			overridden: !variable,
		}
	if (override?.kind === 'off') return { kind: 'off', overridden: !variable }
	const holiday = holidayName(date)
	if (holiday) return { kind: 'holiday', name: holiday }
	if (variable)
		return weekdayOf(date) >= 5
			? { kind: 'off', overridden: false }
			: { kind: 'unplanned' }
	const day = scheduleOn(member, date)[weekdayOf(date)]
	if (day)
		return {
			kind: 'work',
			start: day.start,
			end: day.end,
			lunch: day.lunch ?? null,
			overridden: false,
		}
	const label = blockLabelOn(member, date)
	return label
		? { kind: 'off', overridden: false, label }
		: { kind: 'off', overridden: false }
}

function absenceHalf(
	state: RhState,
	member: Member,
	date: IsoDate,
	part: DayPart,
): HalfDay | null {
	const covering = state.absences.filter(
		(absence) =>
			absence.memberId === member.id &&
			isActiveAbsence(absence) &&
			coversPart(absence, date, part),
	)
	const chosen =
		covering.find((absence) => absence.status === 'approved') ?? covering[0]
	if (!chosen) return null
	const type = findType(state, chosen.typeId)
	if (!type || !countsOn(type.countMode, member, date)) return null
	return {
		kind: 'absence',
		absenceId: chosen.id,
		typeId: chosen.typeId,
		pending: chosen.status === 'pending',
	}
}

export function resolveDay(
	state: RhState,
	member: Member,
	date: IsoDate,
): DayStatus {
	if (!isEmployedOn(member, date))
		return {
			variable: false,
			date,
			morning: { kind: 'inactive' },
			afternoon: { kind: 'inactive' },
			override: null,
		}
	const override = findOverride(state, member.id, date)
	const variable = isVariableOn(member, date)
	const base = baseHalf(member, date, override, variable)
	return {
		variable,
		date,
		override,
		morning: absenceHalf(state, member, date, 'morning') ?? base,
		afternoon: absenceHalf(state, member, date, 'afternoon') ?? base,
	}
}

export type AbsentEntry = {
	member: Member
	typeId: string
	pending: boolean
	part: 'full' | DayPart
}

export type Presence = {
	working: Member[]
	absent: AbsentEntry[]
	off: Member[]
	unplanned: Member[]
}

function hasApprovedExtra(
	state: RhState,
	memberId: string,
	date: IsoDate,
): boolean {
	return state.extras.some(
		(extra) =>
			extra.memberId === memberId &&
			extra.date === date &&
			extra.status === 'approved',
	)
}

export function presenceOn(state: RhState, date: IsoDate): Presence {
	const presence: Presence = { working: [], absent: [], off: [], unplanned: [] }
	for (const member of state.members) {
		if (member.status !== 'active') continue
		const day = resolveDay(state, member, date)
		const halves = [day.morning, day.afternoon]
		if (halves.every((half) => half.kind === 'inactive')) continue
		const absences = halves.flatMap((half) =>
			half.kind === 'absence' ? [half] : [],
		)
		if (absences.length > 0) {
			presence.absent.push({
				member,
				typeId: absences[0].typeId,
				pending: absences.every((half) => half.pending),
				part:
					absences.length === 2
						? 'full'
						: day.morning.kind === 'absence'
							? 'morning'
							: 'afternoon',
			})
		} else if (halves.some((half) => half.kind === 'work')) {
			presence.working.push(member)
		} else if (hasApprovedExtra(state, member.id, date)) {
			presence.working.push(member)
		} else if (halves.some((half) => half.kind === 'unplanned')) {
			presence.unplanned.push(member)
		} else {
			presence.off.push(member)
		}
	}
	return presence
}

export function activePlanningMembers(state: RhState): Member[] {
	return state.members
		.filter((member) => member.status === 'active')
		.sort((a, b) => a.firstName.localeCompare(b.firstName, 'fr'))
}

export function setDayOverride(
	state: RhState,
	_context: ActionContext,
	override: DayOverride,
): RhState {
	return {
		...state,
		overrides: [
			...state.overrides.filter(
				(other) =>
					other.memberId !== override.memberId || other.date !== override.date,
			),
			override,
		],
	}
}

export function clearDayOverride(
	state: RhState,
	memberId: string,
	date: IsoDate,
): RhState {
	return {
		...state,
		overrides: state.overrides.filter(
			(other) => other.memberId !== memberId || other.date !== date,
		),
	}
}
