import { findMember } from '#/pages/hr/lib/rh/absences'
import {
	addMonths,
	isIsoDate,
	startOfMonth,
	WEEKDAY_LABELS,
	weekdayOf,
} from '#/pages/hr/lib/rh/dates'
import {
	equivalentMinutes,
	extraMinutes,
	findSurcharge,
	formatDuration,
	isActiveExtra,
	MAX_RATE,
	suggestSurcharge,
} from '#/pages/hr/lib/rh/extra'
import { holidayName } from '#/pages/hr/lib/rh/holidays'
import { fullName } from '#/pages/hr/lib/rh/members'
import { validateWorkDay } from '#/pages/hr/lib/rh/schedule'
import type {
	ExtraDraft,
	ExtraPeriod,
	ExtraStatus,
	IsoDate,
	LunchBreak,
	RhState,
	SurchargeKind,
} from '#/pages/hr/lib/rh/types'

export const HISTORY_ALL = 'all'

export const CUSTOM_RATE = 'custom'

export type HistoryFilters = {
	query: string
	status: ExtraStatus | typeof HISTORY_ALL
	memberId: string
}

export type Span = {
	start: string
	end: string
	lunch: LunchBreak | null
	rate: number
}

export type Majoration = {
	enabled: boolean
	choice: string
	custom: string
}

export type ExtraForm = {
	memberId: string
	date: IsoDate
	start: string
	end: string
	lunchOn: boolean
	lunch: LunchBreak
	reason: string
	majoration: Majoration
}

export const SURCHARGE_KINDS: readonly {
	value: SurchargeKind
	label: string
}[] = [
	{ value: 'overtime', label: 'Heures supplémentaires' },
	{ value: 'sunday', label: 'Dimanche' },
	{ value: 'holiday', label: 'Jour férié' },
	{ value: 'night', label: 'Nuit' },
	{ value: 'other', label: 'Autre' },
]

export function kindLabel(kind: SurchargeKind): string {
	return SURCHARGE_KINDS.find((entry) => entry.value === kind)?.label ?? 'Autre'
}

export function normalizeText(value: string): string {
	return value
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.trim()
		.toLowerCase()
}

function safeRate(rate: number): number {
	return Number.isFinite(rate) ? rate : 0
}

export function netMinutes(span: Span): number {
	return validateWorkDay({
		start: span.start,
		end: span.end,
		lunch: span.lunch,
	}) === null
		? extraMinutes(span)
		: 0
}

export function paidMinutes(span: Span): number {
	return validateWorkDay({
		start: span.start,
		end: span.end,
		lunch: span.lunch,
	}) === null
		? equivalentMinutes({ ...span, rate: safeRate(span.rate) })
		: 0
}

export function bonusMinutes(span: Span): number {
	return paidMinutes(span) - netMinutes(span)
}

export function describeEquivalent(span: Span): string {
	return `${formatDuration(netMinutes(span))} → ${formatDuration(paidMinutes(span))} payées`
}

export function describeBonus(minutes: number): string {
	return minutes > 0
		? `dont ${formatDuration(minutes)} de majoration`
		: 'Aucune majoration'
}

export function holidayNote(date: IsoDate): string | null {
	if (!isIsoDate(date)) return null
	const holiday = holidayName(date)
	return holiday ? `Jour férié : ${holiday}` : null
}

export function dayCaption(date: IsoDate): string {
	if (!isIsoDate(date)) return ''
	return holidayName(date) ?? WEEKDAY_LABELS[weekdayOf(date)]
}

export function describeHourExample(rate: number): string {
	const paid = Math.round(60 * (1 + safeRate(rate) / 100))
	return `Par exemple, 1 h travaillée sera payée ${formatDuration(paid)}.`
}

export function resolveDate(
	value: string | undefined,
	fallback: IsoDate,
): IsoDate {
	return isIsoDate(value) ? value : fallback
}

export function shiftMonth(month: IsoDate, delta: number): IsoDate {
	return addMonths(startOfMonth(month), delta)
}

export function isRateInvalid(rate: number): boolean {
	return !Number.isFinite(rate) || rate < 0 || rate > MAX_RATE
}

export function faultyField(
	name: string,
	rate: number,
	hasError: boolean,
): 'name' | 'rate' | null {
	if (!hasError) return null
	if (name.trim() === '') return 'name'
	return !Number.isFinite(rate) || rate <= 0 || rate > MAX_RATE
		? 'rate'
		: 'name'
}

export function parseRate(input: string): number {
	const cleaned = input.trim().replace(',', '.')
	return cleaned === '' ? Number.NaN : Number(cleaned)
}

export function suggestedMajoration(state: RhState, date: IsoDate): Majoration {
	const suggestion = isIsoDate(date) ? suggestSurcharge(state, date) : null
	return {
		enabled: suggestion !== null,
		choice: suggestion?.id ?? CUSTOM_RATE,
		custom: '',
	}
}

export function resolveMajoration(
	state: RhState,
	majoration: Majoration,
): { surchargeId: string | null; rate: number } {
	if (!majoration.enabled) return { surchargeId: null, rate: 0 }
	if (majoration.choice === CUSTOM_RATE)
		return { surchargeId: null, rate: parseRate(majoration.custom) }
	const surcharge = findSurcharge(state, majoration.choice)
	return surcharge
		? { surchargeId: surcharge.id, rate: surcharge.rate }
		: { surchargeId: null, rate: 0 }
}

export function buildExtraDraft(state: RhState, form: ExtraForm): ExtraDraft {
	const { surchargeId, rate } = resolveMajoration(state, form.majoration)
	return {
		memberId: form.memberId,
		date: form.date,
		start: form.start,
		end: form.end,
		lunch: form.lunchOn ? form.lunch : null,
		reason: form.reason,
		surchargeId,
		rate,
	}
}

export function canCancel(extra: ExtraPeriod, today: IsoDate): boolean {
	return isActiveExtra(extra) && extra.date >= today
}

export function canCancelOwn(
	extra: ExtraPeriod,
	actorId: string,
	today: IsoDate,
): boolean {
	return (
		actorId !== '' &&
		extra.createdById === actorId &&
		extra.status === 'pending' &&
		extra.date >= today
	)
}

export function sortPending(extras: readonly ExtraPeriod[]): ExtraPeriod[] {
	return [...extras].sort(
		(left, right) =>
			left.date.localeCompare(right.date) ||
			left.start.localeCompare(right.start) ||
			Date.parse(left.createdAt) - Date.parse(right.createdAt) ||
			left.id.localeCompare(right.id),
	)
}

export function filterHistory(
	state: RhState,
	filters: HistoryFilters,
): ExtraPeriod[] {
	const query = normalizeText(filters.query)
	return state.extras
		.filter((extra) => {
			if (filters.status !== HISTORY_ALL && extra.status !== filters.status)
				return false
			if (
				filters.memberId !== HISTORY_ALL &&
				extra.memberId !== filters.memberId
			)
				return false
			if (query === '') return true
			const member = findMember(state, extra.memberId)
			return member ? normalizeText(fullName(member)).includes(query) : false
		})
		.sort(
			(left, right) =>
				right.date.localeCompare(left.date) ||
				right.start.localeCompare(left.start) ||
				Date.parse(right.createdAt) - Date.parse(left.createdAt) ||
				left.id.localeCompare(right.id),
		)
}
