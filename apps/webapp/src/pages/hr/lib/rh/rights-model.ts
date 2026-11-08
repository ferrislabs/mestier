import {
	type BalanceLine,
	type BalancePeriod,
	computeBalance,
	type Movement,
	periodOf,
	validateAdjustment,
} from '#/pages/hr/lib/rh/balances'
import { hireDateOf, leaveDateOf } from '#/pages/hr/lib/rh/contracts'
import {
	formatDaysCompact,
	formatFull,
	formatRange,
	formatShort,
	parseIso,
} from '#/pages/hr/lib/rh/dates'
import type {
	Acquisition,
	AdjustmentDraft,
	IsoDate,
	LeaveAccount,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'

export const LOW_BALANCE_THRESHOLD = 2

const MAX_MONTHLY_RATE = 31
const MAX_ANNUAL_DAYS = 366
const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/

const RATE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 2 })
const DAY = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	year: 'numeric',
	timeZone: 'UTC',
})

export type RightsRow = { member: Member; line: BalanceLine }

export type RightsTotals = {
	count: number
	acquired: number
	taken: number
	planned: number
	pending: number
	balance: number
	available: number
	forecast: number
}

export type Consumption = {
	total: number
	takenShare: number
	plannedShare: number
}

export type MovementDescription = { title: string; detail: string | null }

export const MOVEMENT_NATURES: Record<Movement['kind'], string> = {
	acquisition: 'Acquisition',
	carry_over: 'Report',
	adjustment: 'Régularisation',
	absence: 'Absence',
}

export type AccountSettingsDraft = {
	kind: Acquisition['kind']
	value: number
	periodStartMonth: number
	carryOverMax: number
}

function round2(value: number): number {
	return Math.round(value * 100) / 100
}

export function parseDecimal(text: string): number {
	const cleaned = text.trim().replace(',', '.')
	return cleaned === '' ? Number.NaN : Number(cleaned)
}

export function formatAmount(value: number): string {
	return formatDaysCompact(round2(value)).replace('-', '−')
}

export function formatSigned(value: number): string {
	const rounded = round2(value)
	return rounded > 0 ? `+${formatAmount(rounded)}` : formatAmount(rounded)
}

export function formatDay(iso: IsoDate): string {
	return DAY.format(parseIso(iso))
}

export function describePeriod(period: BalancePeriod): string {
	return `du ${formatFull(period.start)} au ${formatFull(period.end)}`
}

export function describeRule(account: LeaveAccount): string {
	if (account.acquisition.kind === 'annual')
		return `${RATE.format(account.acquisition.days)} j par an`
	const perMonth = account.acquisition.perMonth
	return `${RATE.format(perMonth)} j par mois, soit ${RATE.format(perMonth * 12)} j par an`
}

export function acquiredTotal(line: BalanceLine): number {
	return line.acquired + line.carriedOver + line.adjusted
}

function isPresentOn(member: Member, period: BalancePeriod): boolean {
	const leave = leaveDateOf(member)
	return (
		hireDateOf(member) <= period.end &&
		(leave === null || leave >= period.start)
	)
}

export function rightsRows(
	state: RhState,
	account: LeaveAccount,
	asOf: IsoDate,
): RightsRow[] {
	const period = periodOf(account, asOf)
	return state.members
		.filter(
			(member) => member.status === 'active' && isPresentOn(member, period),
		)
		.map((member) => ({
			member,
			line: computeBalance(state, member, account, asOf),
		}))
}

export function totalsOf(rows: readonly RightsRow[]): RightsTotals {
	return rows.reduce<RightsTotals>(
		(totals, { line }) => ({
			count: totals.count + 1,
			acquired: totals.acquired + acquiredTotal(line),
			taken: totals.taken + line.taken,
			planned: totals.planned + line.planned,
			pending: totals.pending + line.pending,
			balance: totals.balance + line.balance,
			available: totals.available + line.available,
			forecast: totals.forecast + line.forecast,
		}),
		{
			count: 0,
			acquired: 0,
			taken: 0,
			planned: 0,
			pending: 0,
			balance: 0,
			available: 0,
			forecast: 0,
		},
	)
}

export function lowBalanceMembers(
	rows: readonly RightsRow[],
	threshold = LOW_BALANCE_THRESHOLD,
): Member[] {
	return rows
		.filter(({ line }) => line.available < threshold)
		.map(({ member }) => member)
}

export function consumptionOf(line: BalanceLine): Consumption {
	const total = line.forecast + line.taken + line.planned + line.pending
	if (total <= 0) return { total: 0, takenShare: 0, plannedShare: 0 }
	const takenShare = Math.min(1, Math.max(0, line.taken / total))
	const plannedShare = Math.min(
		1 - takenShare,
		Math.max(0, line.planned / total),
	)
	return { total, takenShare, plannedShare }
}

export function orderMovements(movements: readonly Movement[]): Movement[] {
	const rank = (movement: Movement) =>
		movement.kind === 'acquisition' ? 0 : movement.kind === 'carry_over' ? 1 : 2
	return [...movements].sort(
		(left, right) =>
			rank(left) - rank(right) ||
			(rank(left) === 2 ? left.date.localeCompare(right.date) : 0),
	)
}

export function validateAdjustmentDraft(
	state: RhState,
	draft: AdjustmentDraft,
	asOf: IsoDate,
): string[] {
	const base = validateAdjustment(state, draft)
	const account = state.accounts.find((other) => other.id === draft.accountId)
	if (!account) return base
	const period = periodOf(account, asOf)
	const errors = [...base]
	if (!ISO_DATE.test(draft.date)) errors.push('Indique la date du mouvement.')
	else if (draft.date < period.start || draft.date > period.end)
		errors.push(
			`La date doit appartenir à la période (${describePeriod(period)}).`,
		)
	if (base.length === 0 && draft.kind === 'carry_over') {
		const existing = state.adjustments
			.filter(
				(adjustment) =>
					adjustment.memberId === draft.memberId &&
					adjustment.accountId === draft.accountId &&
					adjustment.kind === 'carry_over' &&
					adjustment.date >= period.start &&
					adjustment.date <= period.end,
			)
			.reduce((total, adjustment) => total + adjustment.days, 0)
		if (existing + draft.days > account.carryOverMax)
			errors.push(
				`Ce membre a déjà ${formatAmount(existing)} jours reportés, le total est limité à ${formatAmount(account.carryOverMax)} jours.`,
			)
	}
	return errors
}

export function validateAccountSettings(draft: AccountSettingsDraft): string[] {
	const errors: string[] = []
	const limit = draft.kind === 'monthly' ? MAX_MONTHLY_RATE : MAX_ANNUAL_DAYS
	if (!Number.isFinite(draft.value) || draft.value <= 0)
		errors.push('Indique un nombre de jours acquis strictement positif.')
	else if (draft.value > limit)
		errors.push(
			`L’acquisition ne peut pas dépasser ${limit} jours ${draft.kind === 'monthly' ? 'par mois' : 'par an'}.`,
		)
	if (!Number.isFinite(draft.carryOverMax) || draft.carryOverMax < 0)
		errors.push('Indique un report maximal positif ou nul.')
	if (
		!Number.isInteger(draft.periodStartMonth) ||
		draft.periodStartMonth < 1 ||
		draft.periodStartMonth > 12
	)
		errors.push('Choisis un mois de début de période.')
	return errors
}

export function settingsPatch(
	draft: AccountSettingsDraft,
): Pick<LeaveAccount, 'acquisition' | 'periodStartMonth' | 'carryOverMax'> {
	return {
		acquisition:
			draft.kind === 'monthly'
				? { kind: 'monthly', perMonth: draft.value }
				: { kind: 'annual', days: draft.value },
		periodStartMonth: draft.periodStartMonth,
		carryOverMax: draft.carryOverMax,
	}
}

export function convertAcquisitionValue(
	from: Acquisition['kind'],
	to: Acquisition['kind'],
	value: number,
): number {
	if (from === to) return value
	return round2(from === 'monthly' ? value * 12 : value / 12)
}

export function describeBreakdown(line: BalanceLine): string | null {
	const parts = [
		line.carriedOver !== 0 ? `${formatAmount(line.carriedOver)} de report` : '',
		line.adjusted !== 0 ? `${formatSigned(line.adjusted)} de régul.` : '',
	].filter((part) => part !== '')
	return parts.length > 0 ? `dont ${parts.join(', ')}` : null
}

export function describeMovement(
	state: RhState,
	account: LeaveAccount,
	movement: Movement,
	asOf: IsoDate,
): MovementDescription {
	if (movement.kind === 'acquisition')
		return account.acquisition.kind === 'monthly'
			? {
					title: `Acquisition mensuelle (${RATE.format(account.acquisition.perMonth)} j par mois)`,
					detail: `Cumul au ${formatShort(asOf)}`,
				}
			: {
					title: `Acquisition annuelle (${RATE.format(account.acquisition.days)} j par an)`,
					detail: 'Droit de la période',
				}
	if (movement.kind === 'absence') {
		const absence = state.absences.find((other) => other.id === movement.id)
		return {
			title: movement.label,
			detail: absence ? formatRange(absence.startDate, absence.endDate) : null,
		}
	}
	return { title: movement.label, detail: null }
}
