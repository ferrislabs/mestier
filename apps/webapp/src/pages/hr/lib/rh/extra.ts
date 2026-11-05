import { findMember, findType } from '#/pages/hr/lib/rh/absences'
import { formatRange, formatShort, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { holidayName } from '#/pages/hr/lib/rh/holidays'
import { isEmployedOn, resolveDay } from '#/pages/hr/lib/rh/planning'
import {
	formatWorkDay,
	halfRange,
	minutesOf,
	validateWorkDay,
	workDayMinutes,
} from '#/pages/hr/lib/rh/schedule'
import type {
	ActionContext,
	ExtraDraft,
	ExtraPeriod,
	ExtraStatus,
	IsoDate,
	LunchBreak,
	Member,
	RhState,
	Surcharge,
	SurchargeDraft,
} from '#/pages/hr/lib/rh/types'

export const ACTIVE_EXTRA_STATUSES: readonly ExtraStatus[] = [
	'pending',
	'approved',
]

export const MAX_RATE = 200

export type ExtraSpan = {
	start: string
	end: string
	lunch: LunchBreak | null
	rate: number
}

export function isActiveExtra(extra: Pick<ExtraPeriod, 'status'>): boolean {
	return ACTIVE_EXTRA_STATUSES.includes(extra.status)
}

export function extraMinutes(
	span: Pick<ExtraSpan, 'start' | 'end' | 'lunch'>,
): number {
	return workDayMinutes({ start: span.start, end: span.end, lunch: span.lunch })
}

export function equivalentMinutes(span: ExtraSpan): number {
	return Math.round(extraMinutes(span) * (1 + span.rate / 100))
}

export function formatDuration(minutes: number): string {
	const hours = Math.floor(minutes / 60)
	const rest = minutes % 60
	if (hours === 0) return `${rest} min`
	return rest === 0
		? `${hours} h`
		: `${hours} h ${String(rest).padStart(2, '0')}`
}

export function formatRate(rate: number): string {
	return rate > 0
		? `+${Number.isInteger(rate) ? rate : String(rate).replace('.', ',')} %`
		: 'Sans majoration'
}

export function describeSpan(
	span: Pick<ExtraSpan, 'start' | 'end' | 'lunch'>,
): string {
	return formatWorkDay({ start: span.start, end: span.end, lunch: span.lunch })
}

export function findSurcharge(
	state: RhState,
	surchargeId: string | null,
): Surcharge | undefined {
	return state.surcharges.find((surcharge) => surcharge.id === surchargeId)
}

export function suggestSurcharge(
	state: RhState,
	date: IsoDate,
): Surcharge | null {
	const kind = holidayName(date)
		? 'holiday'
		: weekdayOf(date) === 6
			? 'sunday'
			: 'overtime'
	const usable = state.surcharges.filter((surcharge) => !surcharge.archived)
	return (
		usable.find((surcharge) => surcharge.kind === kind) ??
		usable.find((surcharge) => surcharge.kind === 'overtime') ??
		null
	)
}

export function extrasOn(
	state: RhState,
	memberId: string,
	date: IsoDate,
): ExtraPeriod[] {
	return state.extras
		.filter(
			(extra) =>
				extra.memberId === memberId &&
				extra.date === date &&
				isActiveExtra(extra),
		)
		.sort((a, b) => a.start.localeCompare(b.start))
}

export function pendingExtras(state: RhState): ExtraPeriod[] {
	return state.extras.filter((extra) => extra.status === 'pending')
}

function scheduledSegments(
	state: RhState,
	member: Member,
	date: IsoDate,
): [number, number][] {
	const day = resolveDay(state, member, date)
	const segments: [number, number][] = []
	for (const part of ['morning', 'afternoon'] as const) {
		const half = part === 'morning' ? day.morning : day.afternoon
		if (half.kind !== 'work') continue
		const range = halfRange(
			{ start: half.start, end: half.end, lunch: half.lunch },
			part,
		)
		if (minutesOf(range.end) > minutesOf(range.start))
			segments.push([minutesOf(range.start), minutesOf(range.end)])
	}
	return segments
}

function overlapsMinutes(
	left: [number, number],
	right: [number, number],
): boolean {
	return left[0] < right[1] && left[1] > right[0]
}

export function validateExtra(
	state: RhState,
	draft: ExtraDraft,
	ignoreId?: string,
): string[] {
	const member = findMember(state, draft.memberId)
	if (!member) return ['Choisis un membre.']
	if (!draft.date) return ['Renseigne la date.']
	const spanError = validateWorkDay({
		start: draft.start,
		end: draft.end,
		lunch: draft.lunch,
	})
	if (spanError) return [spanError]
	if (draft.reason.trim() === '') return ['Indique la raison de ces heures.']
	if (!Number.isFinite(draft.rate) || draft.rate < 0 || draft.rate > MAX_RATE)
		return [`La majoration doit être comprise entre 0 et ${MAX_RATE} %.`]
	if (!isEmployedOn(member, draft.date))
		return [`${member.firstName} n’est pas sous contrat ce jour-là.`]
	const day = resolveDay(state, member, draft.date)
	const absent = [day.morning, day.afternoon].find(
		(half) => half.kind === 'absence',
	)
	if (absent && absent.kind === 'absence')
		return [
			`${member.firstName} a une absence ce jour-là (${findType(state, absent.typeId)?.name ?? 'Absence'}).`,
		]
	const span: [number, number] = [minutesOf(draft.start), minutesOf(draft.end)]
	const planned = scheduledSegments(state, member, draft.date)
	if (planned.some((segment) => overlapsMinutes(span, segment)))
		return [
			'Cette période chevauche l’horaire habituel : déclare seulement les heures en plus.',
		]
	const clash = state.extras.find(
		(other) =>
			other.id !== ignoreId &&
			other.memberId === draft.memberId &&
			other.date === draft.date &&
			isActiveExtra(other) &&
			overlapsMinutes(span, [minutesOf(other.start), minutesOf(other.end)]),
	)
	return clash
		? [
				`Une déclaration existe déjà sur cette plage (${describeSpan(clash)}, le ${formatShort(clash.date)}).`,
			]
		: []
}

export function createExtra(
	state: RhState,
	context: ActionContext,
	draft: ExtraDraft,
	approve = false,
): RhState {
	if (validateExtra(state, draft).length > 0) return state
	const extra: ExtraPeriod = {
		...draft,
		reason: draft.reason.trim(),
		id: context.newId(),
		status: approve ? 'approved' : 'pending',
		createdAt: context.now,
		createdById: context.actorId,
		decidedById: approve ? context.actorId : null,
		decidedAt: approve ? context.now : null,
		decisionComment: '',
	}
	return { ...state, extras: [extra, ...state.extras] }
}

export function decideExtra(
	state: RhState,
	context: ActionContext,
	extraId: string,
	decision: 'approved' | 'rejected',
	comment: string,
): RhState {
	return {
		...state,
		extras: state.extras.map((extra) =>
			extra.id === extraId && extra.status === 'pending'
				? {
						...extra,
						status: decision,
						decidedById: context.actorId,
						decidedAt: context.now,
						decisionComment: comment.trim(),
					}
				: extra,
		),
	}
}

export function cancelExtra(
	state: RhState,
	context: ActionContext,
	extraId: string,
): RhState {
	return {
		...state,
		extras: state.extras.map((extra) =>
			extra.id === extraId && isActiveExtra(extra)
				? {
						...extra,
						status: 'cancelled',
						decidedById: context.actorId,
						decidedAt: context.now,
					}
				: extra,
		),
	}
}

export type ExtraSummary = {
	periods: number
	minutes: number
	equivalentMinutes: number
	pendingPeriods: number
	pendingMinutes: number
}

export function summarizeExtras(
	state: RhState,
	month: IsoDate,
	memberId?: string,
): ExtraSummary {
	const key = month.slice(0, 7)
	const inScope = state.extras.filter(
		(extra) =>
			extra.date.slice(0, 7) === key &&
			(memberId === undefined || extra.memberId === memberId),
	)
	const approved = inScope.filter((extra) => extra.status === 'approved')
	const waiting = inScope.filter((extra) => extra.status === 'pending')
	return {
		periods: approved.length,
		minutes: approved.reduce((total, extra) => total + extraMinutes(extra), 0),
		equivalentMinutes: approved.reduce(
			(total, extra) => total + equivalentMinutes(extra),
			0,
		),
		pendingPeriods: waiting.length,
		pendingMinutes: waiting.reduce(
			(total, extra) => total + extraMinutes(extra),
			0,
		),
	}
}

export function validateSurcharge(
	state: RhState,
	draft: SurchargeDraft,
	ignoreId?: string,
): string[] {
	const name = draft.name.trim()
	if (name === '') return ['Donne un nom à cette majoration.']
	if (!Number.isFinite(draft.rate) || draft.rate <= 0 || draft.rate > MAX_RATE)
		return [`Le taux doit être compris entre 1 et ${MAX_RATE} %.`]
	const twin = state.surcharges.some(
		(other) =>
			other.id !== ignoreId &&
			other.name.trim().toLowerCase() === name.toLowerCase() &&
			other.rate === draft.rate,
	)
	return twin ? ['Cette majoration existe déjà.'] : []
}

export function createSurcharge(
	state: RhState,
	context: ActionContext,
	draft: SurchargeDraft,
): RhState {
	if (validateSurcharge(state, draft).length > 0) return state
	return {
		...state,
		surcharges: [
			...state.surcharges,
			{
				...draft,
				name: draft.name.trim(),
				id: context.newId(),
				archived: false,
			},
		],
	}
}

export function updateSurcharge(
	state: RhState,
	surchargeId: string,
	patch: Partial<SurchargeDraft>,
): RhState {
	const current = findSurcharge(state, surchargeId)
	if (!current) return state
	const next = { ...current, ...patch }
	if (validateSurcharge(state, next, surchargeId).length > 0) return state
	return {
		...state,
		surcharges: state.surcharges.map((surcharge) =>
			surcharge.id === surchargeId
				? { ...next, name: next.name.trim() }
				: surcharge,
		),
	}
}

export function setSurchargeArchived(
	state: RhState,
	surchargeId: string,
	archived: boolean,
): RhState {
	return {
		...state,
		surcharges: state.surcharges.map((surcharge) =>
			surcharge.id === surchargeId ? { ...surcharge, archived } : surcharge,
		),
	}
}

export function surchargeUsage(state: RhState, surchargeId: string): number {
	return state.extras.filter((extra) => extra.surchargeId === surchargeId)
		.length
}

export function describeExtraPeriod(extra: ExtraPeriod): string {
	return `${formatRange(extra.date, extra.date)}, ${describeSpan(extra)}`
}
