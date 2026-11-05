import {
	contractOn,
	hireDateOf,
	leaveDateOf,
	scheduleOn,
} from '#/pages/hr/lib/rh/contracts'
import {
	addDays,
	addMonths,
	daysInRange,
	endOfMonth,
	formatLong,
	formatMonth,
	startOfMonth,
	startOfWeek,
	weekdayOf,
} from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	extraMinutes,
	extrasOn,
	formatDuration,
	formatRate,
} from '#/pages/hr/lib/rh/extra'
import { holidayName, isHoliday } from '#/pages/hr/lib/rh/holidays'
import { fullName } from '#/pages/hr/lib/rh/members'
import {
	type DayStatus,
	type HalfDay,
	type Presence,
	resolveDay,
} from '#/pages/hr/lib/rh/planning'
import {
	DEFAULT_LUNCH,
	formatSpan,
	formatWorkDay,
	halfRange,
	minutesOf,
	validateWorkDay,
} from '#/pages/hr/lib/rh/schedule'
import type {
	AbsenceType,
	DayOverride,
	DayOverrideKind,
	DayPart,
	ExtraPeriod,
	IsoDate,
	LunchBreak,
	Member,
	RhState,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export type PlanningView = 'week' | 'month'

export type HalfScope = 'day' | DayPart

export type TypeInfo = Pick<AbsenceType, 'name' | 'hue'>

export type TypeIndex = ReadonlyMap<string, TypeInfo>

export type AbsenceInitial = { memberId?: string; startDate?: IsoDate }

export type ExtraInitial = { memberId?: string; date?: IsoDate }

export type DayLine = { label: string; hours: string | null; text: string }

export type DayForm = {
	kind: DayOverrideKind
	start: string
	end: string
	lunchOn: boolean
	lunchStart: string
	lunchEnd: string
	note: string
}

const FALLBACK_TYPE: TypeInfo = { name: 'Absence', hue: 250 }
const FALLBACK_HOURS = { start: '08:00', end: '17:00' }
const PART_LABELS: Record<DayPart, string> = {
	morning: 'Matin',
	afternoon: 'Après-midi',
}

export function typeIndex(types: readonly AbsenceType[]): TypeIndex {
	return new Map(types.map((type) => [type.id, type]))
}

export function typeInfo(types: TypeIndex, typeId: string): TypeInfo {
	return types.get(typeId) ?? FALLBACK_TYPE
}

export function isWeekend(date: IsoDate): boolean {
	return weekdayOf(date) >= 5
}

export function periodDays(view: PlanningView, date: IsoDate): IsoDate[] {
	if (view === 'month') return daysInRange(startOfMonth(date), endOfMonth(date))
	const first = startOfWeek(date)
	return daysInRange(first, addDays(first, 6))
}

export function shiftPeriod(
	view: PlanningView,
	date: IsoDate,
	direction: -1 | 1,
): IsoDate {
	return view === 'month'
		? addMonths(date, direction)
		: addDays(date, direction * 7)
}

function dayNumber(date: IsoDate): string {
	return String(Number(date.slice(8, 10)))
}

function monthName(date: IsoDate): string {
	return formatMonth(date).replace(/\s*\d{4}$/, '')
}

export function periodLabel(view: PlanningView, date: IsoDate): string {
	if (view === 'month') return formatMonth(date)
	const first = startOfWeek(date)
	const last = addDays(first, 6)
	const year = last.slice(0, 4)
	if (first.slice(0, 7) === last.slice(0, 7))
		return `${dayNumber(first)} – ${dayNumber(last)} ${monthName(last)} ${year}`
	if (first.slice(0, 4) === year)
		return `${dayNumber(first)} ${monthName(first)} – ${dayNumber(last)} ${monthName(last)} ${year}`
	return `${dayNumber(first)} ${monthName(first)} ${first.slice(0, 4)} – ${dayNumber(last)} ${monthName(last)} ${year}`
}

export function dateOnViewChange(
	view: PlanningView,
	date: IsoDate,
	today: IsoDate,
): IsoDate {
	return view === 'week' && today.slice(0, 7) === date.slice(0, 7)
		? today
		: date
}

function searchable(text: string): string {
	return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
}

export function filterMembers(
	members: readonly Member[],
	query: string,
): Member[] {
	const tokens = searchable(query).split(/\s+/).filter(Boolean)
	if (tokens.length === 0) return [...members]
	return members.filter((member) => {
		const haystack = searchable(`${fullName(member)} ${member.jobTitle}`)
		return tokens.every((token) => haystack.includes(token))
	})
}

export function sameHalf(left: HalfDay, right: HalfDay): boolean {
	if (left.kind !== right.kind) return false
	if (left.kind === 'work' && right.kind === 'work')
		return left.start === right.start && left.end === right.end
	if (left.kind === 'absence' && right.kind === 'absence')
		return left.typeId === right.typeId && left.pending === right.pending
	if (left.kind === 'off' && right.kind === 'off')
		return left.label === right.label
	return true
}

function lowerFirst(text: string): string {
	return /^\p{Lu}{2,}/u.test(text)
		? text
		: text.charAt(0).toLowerCase() + text.slice(1)
}

function capitalize(text: string): string {
	return text.charAt(0).toUpperCase() + text.slice(1)
}

export function inactiveReason(member: Member, date: IsoDate): string {
	if (date < hireDateOf(member)) return 'pas encore dans l’équipe'
	const leave = leaveDateOf(member)
	if (leave && date > leave) return 'contrat terminé'
	if (contractOn(member, date) === null) return 'entre deux contrats'
	return 'membre archivé'
}

function spanOf(range: { start: string; end: string }): string {
	return formatSpan(range.start, range.end)
}

function partSpan(day: WorkDay, part: DayPart): string | null {
	const range = halfRange(day, part)
	return minutesOf(range.end) > minutesOf(range.start) ? spanOf(range) : null
}

function plannedDay(member: Member, status: DayStatus): WorkDay | null {
	const { override, date } = status
	if (override?.kind === 'work')
		return {
			start: override.start,
			end: override.end,
			lunch: override.lunch ?? null,
		}
	if (override || holidayName(date) || status.variable) return null
	return scheduleOn(member, date)[weekdayOf(date)]
}

export function describeHalf(
	half: HalfDay,
	scope: HalfScope,
	types: TypeIndex,
	member: Member,
	date: IsoDate,
): string {
	switch (half.kind) {
		case 'work':
			if (scope === 'day') return `travaille ${formatWorkDay(half)}`
			return partSpan(half, scope)
				? `travaille ${partSpan(half, scope)}`
				: 'pas de travail'
		case 'off':
			return half.label ? lowerFirst(half.label) : 'repos'
		case 'unplanned':
			return 'à planifier'
		case 'holiday':
			return `jour férié, ${half.name}`
		case 'absence': {
			const name = lowerFirst(typeInfo(types, half.typeId).name)
			return half.pending ? `${name}, à valider` : name
		}
		case 'inactive':
			return inactiveReason(member, date)
	}
}

export function describeDay(
	status: DayStatus,
	types: TypeIndex,
	member: Member,
): string {
	const { morning, afternoon, date } = status
	if (sameHalf(morning, afternoon))
		return describeHalf(morning, 'day', types, member, date)
	return `matin, ${describeHalf(morning, 'morning', types, member, date)} ; après-midi, ${describeHalf(afternoon, 'afternoon', types, member, date)}`
}

export function overrideNote(status: DayStatus): string | null {
	if (!status.override) return null
	const note = status.override.note.trim()
	if (status.variable) return note ? `Note : ${note}` : null
	return note ? `Jour modifié : ${note}` : 'Jour modifié'
}

export function describeExtras(extras: readonly ExtraPeriod[]): string {
	return extras
		.map((extra) => {
			const rate =
				extra.rate > 0
					? `, majorées de ${formatRate(extra.rate).replace(/^\+/, '')}`
					: ''
			const pending = extra.status === 'pending' ? ', à valider' : ''
			return ` ; heures extra ${describeSpan(extra)}${rate}${pending}`
		})
		.join('')
}

export function cellLabel(
	status: DayStatus,
	types: TypeIndex,
	member: Member,
	extras: readonly ExtraPeriod[] = [],
): string {
	const suffix = status.override && !status.variable ? ', jour modifié' : ''
	return `${fullName(member)}, ${formatLong(status.date)} : ${describeDay(status, types, member)}${suffix}${describeExtras(extras)}`
}

export function extraTotals(extras: readonly ExtraPeriod[]): {
	approved: number
	pending: number
} {
	const sum = (status: ExtraPeriod['status']) =>
		extras
			.filter((extra) => extra.status === status)
			.reduce((total, extra) => total + extraMinutes(extra), 0)
	return { approved: sum('approved'), pending: sum('pending') }
}

export function extraLine(extra: ExtraPeriod): string {
	const pending = extra.status === 'pending' ? ' · à valider' : ''
	return `${describeSpan(extra)} · ${formatDuration(extraMinutes(extra))} · ${formatRate(extra.rate)}${pending}`
}

export function restExtraLabel(extras: readonly ExtraPeriod[]): string | null {
	const spans = extras
		.filter((extra) => extra.status === 'approved')
		.map((extra) => describeSpan(extra))
	return spans.length > 0 ? `Extra ${spans.join(' · ')}` : null
}

export function dayLines(
	status: DayStatus,
	types: TypeIndex,
	member: Member,
): DayLine[] {
	const { morning, afternoon, date } = status
	if (sameHalf(morning, afternoon) && morning.kind !== 'work')
		return [
			{
				label: 'Journée',
				hours: null,
				text: capitalize(describeHalf(morning, 'day', types, member, date)),
			},
		]
	const planned = plannedDay(member, status)
	return (['morning', 'afternoon'] as const).map((part) => {
		const half = part === 'morning' ? morning : afternoon
		const label = PART_LABELS[part]
		if (half.kind === 'work')
			return {
				label,
				hours: null,
				text: partSpan(half, part) ?? 'Pas de travail',
			}
		return {
			label,
			hours:
				half.kind === 'absence' && planned ? partSpan(planned, part) : null,
			text: capitalize(describeHalf(half, part, types, member, date)),
		}
	})
}

export function tooltipLines(
	status: DayStatus,
	types: TypeIndex,
	member: Member,
): DayLine[] {
	const { morning, afternoon, date } = status
	if (sameHalf(morning, afternoon) && morning.kind === 'work')
		return [
			{
				label: 'Journée',
				hours: null,
				text: capitalize(describeHalf(morning, 'day', types, member, date)),
			},
		]
	return dayLines(status, types, member)
}

export function lineTitle(line: DayLine): string {
	return line.hours ? `${line.label} (${line.hours})` : line.label
}

export function shortWork(half: HalfDay, scope: HalfScope): string {
	if (half.kind !== 'work') return ''
	return scope === 'day'
		? formatSpan(half.start, half.end)
		: (partSpan(half, scope) ?? '')
}

export function presentCount(
	state: RhState,
	members: readonly Member[],
	date: IsoDate,
): number {
	return members.filter((member) => {
		const day = resolveDay(state, member, date)
		const halves = [day.morning, day.afternoon]
		if (halves.some((half) => half.kind === 'work')) return true
		if (
			halves.some((half) => half.kind === 'absence' || half.kind === 'inactive')
		)
			return false
		return extrasOn(state, member.id, date).some(
			(extra) => extra.status === 'approved',
		)
	}).length
}

export function presenceSummary(presence: Presence): string {
	const absents = presence.absent.length
	const unplanned =
		presence.unplanned.length > 0
			? ` · ${presence.unplanned.length} à planifier`
			: ''
	return `${presence.working.length} au travail · ${absents} ${absents > 1 ? 'absents' : 'absent'} · ${presence.off.length} en repos${unplanned}`
}

function lunchDefaults(day: WorkDay | null): {
	on: boolean
	lunch: LunchBreak
} {
	if (!day) return { on: true, lunch: DEFAULT_LUNCH }
	return day.lunch
		? { on: true, lunch: day.lunch }
		: { on: false, lunch: DEFAULT_LUNCH }
}

export function initialDayForm(
	member: Member,
	date: IsoDate,
	override: DayOverride | null,
): DayForm {
	const contract = contractOn(member, date)
	const variableDay =
		contract?.schedule.kind === 'variable' ? contract.schedule.defaultDay : null
	const scheduled = scheduleOn(member, date)[weekdayOf(date)]
	const source = scheduled ?? variableDay
	const hours = source ?? FALLBACK_HOURS
	const pause = lunchDefaults(source)
	const base = {
		start: hours.start,
		end: hours.end,
		lunchOn: pause.on,
		lunchStart: pause.lunch.start,
		lunchEnd: pause.lunch.end,
	}
	if (override?.kind === 'work')
		return {
			kind: 'work',
			start: override.start || base.start,
			end: override.end || base.end,
			lunchOn: override.lunch != null,
			lunchStart: override.lunch?.start ?? base.lunchStart,
			lunchEnd: override.lunch?.end ?? base.lunchEnd,
			note: override.note,
		}
	if (override) return { ...base, kind: 'off', note: override.note }
	return {
		...base,
		kind: scheduled && !isHoliday(date) ? 'work' : 'off',
		note: '',
	}
}

function toWorkDay(form: DayForm): WorkDay {
	return {
		start: form.start,
		end: form.end,
		lunch: form.lunchOn ? { start: form.lunchStart, end: form.lunchEnd } : null,
	}
}

export function dayFormError(form: DayForm): string | null {
	return form.kind === 'off' ? null : validateWorkDay(toWorkDay(form))
}

export function dayFormFault(form: DayForm): 'day' | 'lunch' | null {
	if (form.kind === 'off') return null
	if (validateWorkDay({ start: form.start, end: form.end, lunch: null }))
		return 'day'
	return validateWorkDay(toWorkDay(form)) ? 'lunch' : null
}

export function isDayFormDirty(initial: DayForm, form: DayForm): boolean {
	if (initial.kind !== form.kind) return true
	if (initial.note.trim() !== form.note.trim()) return true
	if (form.kind !== 'work') return false
	if (initial.start !== form.start || initial.end !== form.end) return true
	if (initial.lunchOn !== form.lunchOn) return true
	return (
		form.lunchOn &&
		(initial.lunchStart !== form.lunchStart ||
			initial.lunchEnd !== form.lunchEnd)
	)
}

export function toOverride(
	memberId: string,
	date: IsoDate,
	form: DayForm,
): DayOverride {
	const works = form.kind === 'work'
	return {
		memberId,
		date,
		kind: form.kind,
		start: works ? form.start : '',
		end: works ? form.end : '',
		lunch: works ? toWorkDay(form).lunch : null,
		note: form.note.trim(),
	}
}
