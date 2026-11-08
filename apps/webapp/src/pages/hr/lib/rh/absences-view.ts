import {
	absenceShares,
	findMember,
	findType,
	isActiveAbsence,
} from '#/pages/hr/lib/rh/absences'
import { computeBalance, periodOf } from '#/pages/hr/lib/rh/balances'
import { formatDaysCompact, parseIso, toIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type {
	Absence,
	AbsenceSpanInput,
	AbsenceStatus,
	CountMode,
	DayPart,
	IsoDate,
	LeaveAccount,
	RhState,
} from '#/pages/hr/lib/rh/types'

type Span = {
	startDate: IsoDate
	startPart: DayPart
	endDate: IsoDate
	endPart: DayPart
}

export const HISTORY_ALL = 'all'

const ABSENCE_STATUSES: readonly AbsenceStatus[] = [
	'pending',
	'approved',
	'rejected',
	'cancelled',
]

export function isAbsenceStatus(value: unknown): value is AbsenceStatus {
	return ABSENCE_STATUSES.some((status) => status === value)
}

export type HistoryFilters = {
	query: string
	status: AbsenceStatus | typeof HISTORY_ALL
	memberId: string
	typeId: string
}

export type CountModeOption = {
	value: CountMode
	label: string
	short: string
	help: string
}

export const COUNT_MODES: readonly CountModeOption[] = [
	{
		value: 'working',
		label: 'Jours ouvrés',
		short: 'en jours ouvrés',
		help: 'Les jours travaillés par le membre, hors jours fériés.',
	},
	{
		value: 'workable',
		label: 'Jours ouvrables',
		short: 'en jours ouvrables',
		help: 'Du lundi au samedi, hors dimanches et jours fériés.',
	},
	{
		value: 'calendar',
		label: 'Jours calendaires',
		short: 'en jours calendaires',
		help: 'Tous les jours, week-ends et jours fériés compris.',
	},
]

export function countModeOption(mode: CountMode): CountModeOption {
	return COUNT_MODES.find((option) => option.value === mode) ?? COUNT_MODES[0]
}

export const TYPE_HUES: readonly { hue: number; name: string }[] = [
	{ hue: 25, name: 'Corail' },
	{ hue: 60, name: 'Ocre' },
	{ hue: 100, name: 'Olive' },
	{ hue: 155, name: 'Vert' },
	{ hue: 190, name: 'Turquoise' },
	{ hue: 215, name: 'Bleu ciel' },
	{ hue: 250, name: 'Bleu' },
	{ hue: 285, name: 'Violet' },
	{ hue: 320, name: 'Prune' },
	{ hue: 355, name: 'Rose' },
]

export function formatDayCount(days: number): string {
	const label = formatDaysCompact(days)
	return `${label} ${Math.abs(days) < 2 ? 'jour' : 'jours'}`
}

export function formatDayAmount(days: number): string {
	return `${formatDaysCompact(days).replace('-', '−')} j`
}

const WEEKDAY = new Intl.DateTimeFormat('fr-FR', {
	weekday: 'long',
	timeZone: 'UTC',
})
const MONTH = new Intl.DateTimeFormat('fr-FR', {
	month: 'long',
	timeZone: 'UTC',
})

function dayLabel(iso: IsoDate): string {
	const day = Number(iso.slice(8, 10))
	return day === 1 ? '1er' : String(day)
}

function weekdayLabel(iso: IsoDate): string {
	return WEEKDAY.format(parseIso(iso))
}

function monthLabel(iso: IsoDate): string {
	return MONTH.format(parseIso(iso))
}

function sameMonth(start: IsoDate, end: IsoDate): boolean {
	return start.slice(0, 7) === end.slice(0, 7)
}

function sameYear(start: IsoDate, end: IsoDate): boolean {
	return start.slice(0, 4) === end.slice(0, 4)
}

export function describeSpan(span: Span): string {
	const { startDate: start, endDate: end } = span
	const first = `${weekdayLabel(start)} ${dayLabel(start)}`
	if (start === end) return `${first} ${monthLabel(start)}`
	const last = `${weekdayLabel(end)} ${dayLabel(end)} ${monthLabel(end)}`
	if (!sameYear(start, end))
		return `${first} ${monthLabel(start)} ${start.slice(0, 4)} au ${last} ${end.slice(0, 4)}`
	if (sameMonth(start, end)) return `${first} au ${last}`
	return `${first} ${monthLabel(start)} au ${last}`
}

export function describeDateRange(start: IsoDate, end: IsoDate): string {
	if (start === end) return `le ${dayLabel(start)} ${monthLabel(start)}`
	if (sameMonth(start, end))
		return `du ${dayLabel(start)} au ${dayLabel(end)} ${monthLabel(end)}`
	return `du ${dayLabel(start)} ${monthLabel(start)} au ${dayLabel(end)} ${monthLabel(end)}`
}

export function partsNote(span: Span): string | null {
	if (span.startDate === span.endDate) {
		if (span.startPart === 'morning' && span.endPart === 'morning')
			return 'Matin uniquement'
		if (span.startPart === 'afternoon' && span.endPart === 'afternoon')
			return 'Après-midi uniquement'
		return null
	}
	const notes = [
		span.startPart === 'afternoon' ? 'Début l’après-midi' : null,
		span.endPart === 'morning' ? 'fin à midi' : null,
	].filter((note): note is string => note !== null)
	if (notes.length === 0) return null
	const joined = notes.join(', ')
	return joined.charAt(0).toUpperCase() + joined.slice(1)
}

export type BalanceImpact = {
	account: LeaveAccount
	days: number
	before: number
	after: number
	shortfall: number
}

export function balanceImpact(
	state: RhState,
	span: AbsenceSpanInput,
	today: IsoDate,
	alreadyCounted: boolean,
): BalanceImpact | null {
	const member = findMember(state, span.memberId)
	const type = findType(state, span.typeId)
	const account = state.accounts.find((other) => other.id === type?.accountId)
	if (!member || !type || !account) return null
	const current = periodOf(account, today)
	const inCurrent =
		span.startDate >= current.start && span.startDate <= current.end
	const asOf = inCurrent ? today : span.startDate
	const period = periodOf(account, asOf)
	const days = absenceShares(span, member, type)
		.filter((share) => share.date >= period.start && share.date <= period.end)
		.reduce((total, share) => total + share.value, 0)
	const { available } = computeBalance(state, member, account, asOf)
	const before = alreadyCounted ? available + days : available
	const after = before - days
	return { account, days, before, after, shortfall: Math.max(0, -after) }
}

const ISO_SHAPE = /^\d{4}-\d{2}-\d{2}$/

export function resolveStartDate(
	value: string | undefined,
	fallback: IsoDate,
): IsoDate {
	return value && ISO_SHAPE.test(value) && toIso(parseIso(value)) === value
		? value
		: fallback
}

export function canCancel(absence: Absence, today: IsoDate): boolean {
	return isActiveAbsence(absence) && absence.endDate >= today
}

export function canCancelOwn(
	absence: Absence,
	actorId: string,
	today: IsoDate,
): boolean {
	return (
		actorId !== '' &&
		absence.createdById === actorId &&
		absence.status === 'pending' &&
		absence.endDate >= today
	)
}

export function normalizeText(value: string): string {
	return value
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.trim()
		.toLowerCase()
}

export function sortPending(absences: readonly Absence[]): Absence[] {
	return [...absences].sort(
		(left, right) =>
			left.startDate.localeCompare(right.startDate) ||
			Date.parse(left.createdAt) - Date.parse(right.createdAt) ||
			left.id.localeCompare(right.id),
	)
}

export function filterHistory(
	state: RhState,
	filters: HistoryFilters,
): Absence[] {
	const query = normalizeText(filters.query)
	return state.absences
		.filter((absence) => {
			if (filters.status !== HISTORY_ALL && absence.status !== filters.status)
				return false
			if (
				filters.memberId !== HISTORY_ALL &&
				absence.memberId !== filters.memberId
			)
				return false
			if (filters.typeId !== HISTORY_ALL && absence.typeId !== filters.typeId)
				return false
			if (query === '') return true
			const member = findMember(state, absence.memberId)
			return member ? normalizeText(fullName(member)).includes(query) : false
		})
		.sort(
			(left, right) =>
				right.startDate.localeCompare(left.startDate) ||
				Date.parse(right.createdAt) - Date.parse(left.createdAt) ||
				left.id.localeCompare(right.id),
		)
}

export function validateTypeName(
	state: RhState,
	name: string,
	ignoreId?: string,
): string | null {
	const normalized = normalizeText(name)
	if (normalized === '') return 'Le nom est obligatoire.'
	const clash = state.absenceTypes.some(
		(type) => type.id !== ignoreId && normalizeText(type.name) === normalized,
	)
	return clash ? 'Un motif porte déjà ce nom.' : null
}
