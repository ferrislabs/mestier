import { scheduleOn } from '#/pages/hr/lib/rh/contracts'
import { daysInRange, formatRange, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { isHoliday } from '#/pages/hr/lib/rh/holidays'
import type {
	Absence,
	AbsenceSpanInput,
	AbsenceStatus,
	AbsenceType,
	CountMode,
	DayPart,
	IsoDate,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type DayShare = { date: IsoDate; value: number }

type Span = Pick<Absence, 'startDate' | 'startPart' | 'endDate' | 'endPart'>

export const ACTIVE_STATUSES: readonly AbsenceStatus[] = ['pending', 'approved']

export function isActiveAbsence(absence: Absence): boolean {
	return ACTIVE_STATUSES.includes(absence.status)
}

export function countsOn(
	mode: CountMode,
	member: Member,
	date: IsoDate,
): boolean {
	if (mode === 'calendar') return true
	if (isHoliday(date)) return false
	const weekday = weekdayOf(date)
	return mode === 'workable'
		? weekday !== 6
		: scheduleOn(member, date)[weekday] !== null
}

export function coversPart(span: Span, date: IsoDate, part: DayPart): boolean {
	if (date < span.startDate || date > span.endDate) return false
	if (date === span.startDate && span.startPart === 'afternoon')
		return part === 'afternoon'
	if (date === span.endDate && span.endPart === 'morning')
		return part === 'morning'
	return true
}

export function absenceShares(
	span: Span,
	member: Member,
	type: Pick<AbsenceType, 'countMode'>,
): DayShare[] {
	return daysInRange(span.startDate, span.endDate).flatMap((date) => {
		if (!countsOn(type.countMode, member, date)) return []
		const morning = coversPart(span, date, 'morning') ? 0.5 : 0
		const afternoon = coversPart(span, date, 'afternoon') ? 0.5 : 0
		const value = morning + afternoon
		return value > 0 ? [{ date, value }] : []
	})
}

export function absenceDays(
	span: Span,
	member: Member,
	type: Pick<AbsenceType, 'countMode'>,
): number {
	return absenceShares(span, member, type).reduce(
		(total, share) => total + share.value,
		0,
	)
}

function slotsOf(span: Span): Set<string> {
	const slots = new Set<string>()
	for (const date of daysInRange(span.startDate, span.endDate)) {
		for (const part of ['morning', 'afternoon'] as const) {
			if (coversPart(span, date, part)) slots.add(`${date}:${part}`)
		}
	}
	return slots
}

function overlaps(left: Span, right: Span): boolean {
	const rightSlots = slotsOf(right)
	for (const slot of slotsOf(left)) if (rightSlots.has(slot)) return true
	return false
}

export function findType(
	state: RhState,
	typeId: string,
): AbsenceType | undefined {
	return state.absenceTypes.find((type) => type.id === typeId)
}

export function findMember(
	state: RhState,
	memberId: string | null,
): Member | undefined {
	return state.members.find((member) => member.id === memberId)
}

export function validateAbsence(
	state: RhState,
	draft: AbsenceSpanInput,
	ignoreId?: string,
): string[] {
	const member = findMember(state, draft.memberId)
	const type = findType(state, draft.typeId)
	if (!member) return ['Choisis un membre.']
	if (!type) return ['Choisis un motif d’absence.']
	if (!draft.startDate || !draft.endDate)
		return ['Renseigne la date de début et la date de fin.']
	if (
		draft.endDate < draft.startDate ||
		(draft.endDate === draft.startDate &&
			draft.startPart === 'afternoon' &&
			draft.endPart === 'morning')
	)
		return ['La fin de l’absence précède son début.']
	if (absenceDays(draft, member, type) === 0)
		return ['Cette période ne contient aucun jour décompté pour ce motif.']
	const clash = state.absences.find(
		(other) =>
			other.id !== ignoreId &&
			other.memberId === draft.memberId &&
			isActiveAbsence(other) &&
			overlaps(other, draft),
	)
	if (clash) {
		const clashType = findType(state, clash.typeId)
		return [
			`${member.firstName} a déjà une absence sur cette période (${clashType?.name ?? 'Absence'}, ${formatRange(clash.startDate, clash.endDate)}).`,
		]
	}
	return []
}

export type TeamConflict = { member: Member; absence: Absence }

export function teamConflicts(
	state: RhState,
	draft: AbsenceSpanInput,
): TeamConflict[] {
	return state.absences.flatMap((other) => {
		if (other.memberId === draft.memberId || !isActiveAbsence(other)) return []
		const member = findMember(state, other.memberId)
		return member && member.status === 'active' && overlaps(other, draft)
			? [{ member, absence: other }]
			: []
	})
}

export function pendingAbsences(state: RhState): Absence[] {
	return state.absences.filter((absence) => absence.status === 'pending')
}

export function typeUsage(state: RhState, typeId: string): number {
	return state.absences.filter((absence) => absence.typeId === typeId).length
}
