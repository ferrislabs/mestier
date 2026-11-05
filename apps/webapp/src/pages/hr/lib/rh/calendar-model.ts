import { findType, isActiveAbsence } from '#/pages/hr/lib/rh/absences'
import { weekOn } from '#/pages/hr/lib/rh/contract-schedule'
import { contractOn } from '#/pages/hr/lib/rh/contracts'
import {
	addDays,
	addMonths,
	daysInRange,
	endOfMonth,
	formatLong,
	formatMonth,
	startOfMonth,
	startOfWeek,
} from '#/pages/hr/lib/rh/dates'
import { extraMinutes, extrasOn, formatDuration } from '#/pages/hr/lib/rh/extra'
import { type HalfDay, resolveDay } from '#/pages/hr/lib/rh/planning'
import {
	formatSpan,
	formatWorkDay,
	halfRange,
} from '#/pages/hr/lib/rh/schedule'
import type {
	Absence,
	DayPart,
	ExtraPeriod,
	IsoDate,
	Member,
	RhState,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export type HalfModel =
	| { kind: 'work'; range: string }
	| { kind: 'off'; label?: string }
	| { kind: 'unplanned' }
	| { kind: 'holiday'; name: string }
	| { kind: 'absence'; typeId: string; absenceId: string; pending: boolean }
	| { kind: 'inactive' }

export type CellModel = {
	date: IsoDate
	morning: HalfModel
	afternoon: HalfModel
	split: boolean
	compactRange: string | null
	dayRange: string | null
	lunch: string | null
	overridden: boolean
	variable: boolean
	planned: boolean
	note: string
	extraApproved: number
	extraPending: number
}

export type HalfDetail = {
	part: 'Journée' | 'Matin' | 'Après-midi'
	model: HalfModel
	absence: Absence | null
}

export type DayDetails = {
	title: string
	halves: HalfDetail[]
	dayRange: string | null
	lunch: string | null
	note: string | null
	sequence: string | null
	variable: { defaultDay: WorkDay; planned: boolean } | null
	extras: ExtraPeriod[]
}

export function monthWeeks(month: IsoDate): IsoDate[][] {
	const first = startOfMonth(month)
	const days = daysInRange(
		startOfWeek(first),
		addDays(startOfWeek(endOfMonth(first)), 6),
	)
	return Array.from({ length: days.length / 7 }, (_, week) =>
		days.slice(week * 7, week * 7 + 7),
	)
}

export function shiftMonth(month: IsoDate, delta: number): IsoDate {
	return addMonths(month, delta)
}

export function monthLabel(month: IsoDate): string {
	const label = formatMonth(month)
	return label.charAt(0).toUpperCase() + label.slice(1)
}

export function isSameMonth(left: IsoDate, right: IsoDate): boolean {
	return left.slice(0, 7) === right.slice(0, 7)
}

function halfModel(half: HalfDay, part: DayPart): HalfModel {
	switch (half.kind) {
		case 'work': {
			const range = halfRange(
				{ start: half.start, end: half.end, lunch: half.lunch },
				part,
			)
			return { kind: 'work', range: formatSpan(range.start, range.end) }
		}
		case 'off':
			return half.label ? { kind: 'off', label: half.label } : { kind: 'off' }
		case 'unplanned':
			return { kind: 'unplanned' }
		case 'holiday':
			return { kind: 'holiday', name: half.name }
		case 'absence':
			return {
				kind: 'absence',
				typeId: half.typeId,
				absenceId: half.absenceId,
				pending: half.pending,
			}
		case 'inactive':
			return { kind: 'inactive' }
	}
}

function sameModel(left: HalfModel, right: HalfModel): boolean {
	if (left.kind !== right.kind) return false
	if (left.kind === 'absence' && right.kind === 'absence')
		return left.typeId === right.typeId && left.pending === right.pending
	if (left.kind === 'holiday' && right.kind === 'holiday')
		return left.name === right.name
	if (left.kind === 'off' && right.kind === 'off')
		return left.label === right.label
	return true
}

function minutesOfExtras(
	extras: readonly ExtraPeriod[],
	status: ExtraPeriod['status'],
): number {
	return extras
		.filter((extra) => extra.status === status)
		.reduce((total, extra) => total + extraMinutes(extra), 0)
}

export function cellModel(
	state: RhState,
	member: Member,
	date: IsoDate,
): CellModel {
	const status = resolveDay(state, member, date)
	const morning = halfModel(status.morning, 'morning')
	const afternoon = halfModel(status.afternoon, 'afternoon')
	const extras = extrasOn(state, member.id, date)
	const work = status.morning.kind === 'work' ? status.morning : null
	const wholeDay =
		status.morning.kind === 'work' && status.afternoon.kind === 'work'
	return {
		date,
		morning,
		afternoon,
		split: !sameModel(morning, afternoon),
		compactRange: wholeDay && work ? formatSpan(work.start, work.end) : null,
		dayRange: wholeDay && work ? formatWorkDay(work) : null,
		lunch: work?.lunch ? formatSpan(work.lunch.start, work.lunch.end) : null,
		overridden: status.override !== null && !status.variable,
		variable: status.variable,
		planned: status.variable && status.override !== null,
		note: status.override?.note ?? '',
		extraApproved: minutesOfExtras(extras, 'approved'),
		extraPending: minutesOfExtras(extras, 'pending'),
	}
}

export function extraBadge(minutes: number): string {
	return `+${formatDuration(minutes)}`
}

function describeHalf(
	state: RhState,
	half: HalfModel,
	dayRange: string | null,
): string {
	switch (half.kind) {
		case 'work':
			return `au travail ${dayRange ?? half.range}`
		case 'off':
			return half.label ?? 'repos'
		case 'unplanned':
			return 'à planifier'
		case 'holiday':
			return `férié (${half.name})`
		case 'absence':
			return `${findType(state, half.typeId)?.name ?? 'Absence'}${half.pending ? ', à valider' : ''}`
		case 'inactive':
			return 'hors contrat'
	}
}

export function cellLabel(state: RhState, cell: CellModel): string {
	const day = cell.split
		? `matin ${describeHalf(state, cell.morning, null)}, après-midi ${describeHalf(state, cell.afternoon, null)}`
		: describeHalf(state, cell.morning, cell.dayRange)
	const extras = [
		cell.extraApproved > 0
			? `${formatDuration(cell.extraApproved)} validées`
			: null,
		cell.extraPending > 0
			? `${formatDuration(cell.extraPending)} en attente`
			: null,
	].filter(Boolean)
	return [
		`${formatLong(cell.date)} : ${day}`,
		extras.length > 0 ? `heures extra : ${extras.join(' et ')}` : null,
		cell.overridden ? 'jour modifié' : null,
	]
		.filter(Boolean)
		.join(', ')
}

function detailOf(state: RhState, part: HalfDetail['part'], model: HalfModel) {
	return {
		part,
		model,
		absence:
			model.kind === 'absence'
				? (state.absences.find(
						(absence) =>
							absence.id === model.absenceId && isActiveAbsence(absence),
					) ?? null)
				: null,
	}
}

export function dayDetails(
	state: RhState,
	member: Member,
	date: IsoDate,
): DayDetails {
	const cell = cellModel(state, member, date)
	const contract = contractOn(member, date)
	const sequenceLabel = contract ? weekOn(contract.schedule, date).label : null
	return {
		title: formatLong(date),
		halves: cell.split
			? [
					detailOf(state, 'Matin', cell.morning),
					detailOf(state, 'Après-midi', cell.afternoon),
				]
			: [detailOf(state, 'Journée', cell.morning)],
		dayRange: cell.dayRange,
		lunch: cell.lunch,
		note: cell.overridden
			? cell.note || 'Jour modifié'
			: cell.planned && cell.note
				? cell.note
				: null,
		sequence: contract?.schedule.kind === 'rotation' ? sequenceLabel : null,
		variable:
			contract?.schedule.kind === 'variable'
				? { defaultDay: contract.schedule.defaultDay, planned: cell.planned }
				: null,
		extras: extrasOn(state, member.id, date),
	}
}
