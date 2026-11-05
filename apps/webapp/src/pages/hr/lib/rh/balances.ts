import { absenceShares, findType } from '#/pages/hr/lib/rh/absences'
import { hireDateOf } from '#/pages/hr/lib/rh/contracts'
import {
	addDays,
	addMonths,
	formatDaysCompact,
	startOfMonth,
} from '#/pages/hr/lib/rh/dates'
import type {
	Absence,
	AbsenceStatus,
	ActionContext,
	AdjustmentDraft,
	IsoDate,
	LeaveAccount,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'

export const DEFAULT_ACCOUNT_ID = 'cp'

export type BalancePeriod = { start: IsoDate; end: IsoDate }

export type BalanceLine = {
	accountId: string
	period: BalancePeriod
	acquired: number
	carriedOver: number
	adjusted: number
	taken: number
	planned: number
	pending: number
	balance: number
	available: number
	forecast: number
}

export type MovementKind =
	| 'acquisition'
	| 'carry_over'
	| 'adjustment'
	| 'absence'

export type Movement = {
	id: string
	kind: MovementKind
	date: IsoDate
	label: string
	days: number
	status: AbsenceStatus | null
}

export function periodOf(account: LeaveAccount, asOf: IsoDate): BalancePeriod {
	const year = Number(asOf.slice(0, 4))
	const month = Number(asOf.slice(5, 7))
	const startYear = month >= account.periodStartMonth ? year : year - 1
	const start = `${startYear}-${String(account.periodStartMonth).padStart(2, '0')}-01`
	return { start, end: addDays(addMonths(start, 12), -1) }
}

function roundHalf(value: number): number {
	return Math.round(value * 2) / 2
}

function monthsBetween(from: IsoDate, to: IsoDate): number {
	return (
		(Number(to.slice(0, 4)) - Number(from.slice(0, 4))) * 12 +
		Number(to.slice(5, 7)) -
		Number(from.slice(5, 7)) +
		1
	)
}

export function acquiredAt(
	account: LeaveAccount,
	member: Member,
	period: BalancePeriod,
	asOf: IsoDate,
): number {
	const from = startOfMonth(
		hireDateOf(member) > period.start ? hireDateOf(member) : period.start,
	)
	const until = asOf < period.end ? asOf : period.end
	if (until < from) return 0
	const elapsed = Math.min(12, monthsBetween(from, until))
	const employed = Math.min(12, monthsBetween(from, period.end))
	if (account.acquisition.kind === 'monthly')
		return roundHalf(account.acquisition.perMonth * elapsed)
	return roundHalf((account.acquisition.days * employed) / 12)
}

function accountAbsences(
	state: RhState,
	member: Member,
	account: LeaveAccount,
	statuses: readonly AbsenceStatus[],
): Absence[] {
	return state.absences.filter(
		(absence) =>
			absence.memberId === member.id &&
			statuses.includes(absence.status) &&
			findType(state, absence.typeId)?.accountId === account.id,
	)
}

function sharesInPeriod(
	state: RhState,
	member: Member,
	absence: Absence,
	period: BalancePeriod,
) {
	const type = findType(state, absence.typeId)
	return type
		? absenceShares(absence, member, type).filter(
				(share) => share.date >= period.start && share.date <= period.end,
			)
		: []
}

function sumShares(
	state: RhState,
	member: Member,
	absences: readonly Absence[],
	period: BalancePeriod,
	keep: (date: IsoDate) => boolean,
): number {
	return absences
		.flatMap((absence) => sharesInPeriod(state, member, absence, period))
		.filter((share) => keep(share.date))
		.reduce((total, share) => total + share.value, 0)
}

export function computeBalance(
	state: RhState,
	member: Member,
	account: LeaveAccount,
	asOf: IsoDate,
): BalanceLine {
	const period = periodOf(account, asOf)
	const inPeriod = state.adjustments.filter(
		(adjustment) =>
			adjustment.memberId === member.id &&
			adjustment.accountId === account.id &&
			adjustment.date >= period.start &&
			adjustment.date <= period.end,
	)
	const carriedOver = inPeriod
		.filter((adjustment) => adjustment.kind === 'carry_over')
		.reduce((total, adjustment) => total + adjustment.days, 0)
	const adjusted = inPeriod
		.filter((adjustment) => adjustment.kind === 'adjustment')
		.reduce((total, adjustment) => total + adjustment.days, 0)
	const acquired = acquiredAt(account, member, period, asOf)
	const acquiredAtEnd = acquiredAt(account, member, period, period.end)
	const approved = accountAbsences(state, member, account, ['approved'])
	const waiting = accountAbsences(state, member, account, ['pending'])
	const taken = sumShares(
		state,
		member,
		approved,
		period,
		(date) => date <= asOf,
	)
	const planned = sumShares(
		state,
		member,
		approved,
		period,
		(date) => date > asOf,
	)
	const pending = sumShares(state, member, waiting, period, () => true)
	const balance = acquired + carriedOver + adjusted - taken
	return {
		accountId: account.id,
		period,
		acquired,
		carriedOver,
		adjusted,
		taken,
		planned,
		pending,
		balance,
		available: balance - planned - pending,
		forecast:
			acquiredAtEnd + carriedOver + adjusted - taken - planned - pending,
	}
}

export function movementsFor(
	state: RhState,
	member: Member,
	account: LeaveAccount,
	asOf: IsoDate,
): Movement[] {
	const period = periodOf(account, asOf)
	const adjustments = state.adjustments
		.filter(
			(adjustment) =>
				adjustment.memberId === member.id &&
				adjustment.accountId === account.id &&
				adjustment.date >= period.start &&
				adjustment.date <= period.end,
		)
		.map(
			(adjustment): Movement => ({
				id: adjustment.id,
				kind: adjustment.kind,
				date: adjustment.date,
				label:
					adjustment.reason ||
					(adjustment.kind === 'carry_over' ? 'Report' : 'Régularisation'),
				days: adjustment.days,
				status: null,
			}),
		)
	const absences = accountAbsences(state, member, account, [
		'approved',
		'pending',
	]).flatMap((absence): Movement[] => {
		const days = sharesInPeriod(state, member, absence, period).reduce(
			(total, share) => total + share.value,
			0,
		)
		return days === 0
			? []
			: [
					{
						id: absence.id,
						kind: 'absence',
						date: absence.startDate,
						label: findType(state, absence.typeId)?.name ?? 'Absence',
						days: -days,
						status: absence.status,
					},
				]
	})
	const acquired = acquiredAt(account, member, period, asOf)
	const acquisition: Movement[] =
		acquired === 0
			? []
			: [
					{
						id: `acquisition-${account.id}-${member.id}`,
						kind: 'acquisition',
						date: period.start,
						label:
							account.acquisition.kind === 'monthly'
								? `Acquisition mensuelle (${formatDaysCompact(account.acquisition.perMonth)} j par mois)`
								: 'Acquisition annuelle',
						days: acquired,
						status: null,
					},
				]
	return [...acquisition, ...adjustments, ...absences].sort((a, b) =>
		b.date.localeCompare(a.date),
	)
}

export function validateAdjustment(
	state: RhState,
	draft: AdjustmentDraft,
): string[] {
	const account = state.accounts.find((other) => other.id === draft.accountId)
	if (!account) return ['Choisis un compte.']
	if (!state.members.some((member) => member.id === draft.memberId))
		return ['Choisis un membre.']
	if (!Number.isFinite(draft.days) || draft.days === 0)
		return ['Indique un nombre de jours différent de zéro.']
	if (draft.kind === 'carry_over' && draft.days < 0)
		return ['Un report est toujours positif.']
	if (draft.kind === 'carry_over' && draft.days > account.carryOverMax)
		return [
			`Le report est limité à ${formatDaysCompact(account.carryOverMax)} jours sur ce compte.`,
		]
	if (draft.reason.trim() === '' && draft.kind === 'adjustment')
		return ['Indique le motif de la régularisation.']
	return []
}

export function addAdjustment(
	state: RhState,
	context: ActionContext,
	draft: AdjustmentDraft,
): RhState {
	if (validateAdjustment(state, draft).length > 0) return state
	return {
		...state,
		adjustments: [
			...state.adjustments,
			{
				...draft,
				reason: draft.reason.trim(),
				id: context.newId(),
				authorId: context.actorId,
			},
		],
	}
}

export function removeAdjustment(
	state: RhState,
	adjustmentId: string,
): RhState {
	return {
		...state,
		adjustments: state.adjustments.filter(
			(adjustment) => adjustment.id !== adjustmentId,
		),
	}
}

export function updateAccount(
	state: RhState,
	accountId: string,
	patch: Partial<Omit<LeaveAccount, 'id'>>,
): RhState {
	return {
		...state,
		accounts: state.accounts.map((account) =>
			account.id === accountId ? { ...account, ...patch } : account,
		),
	}
}

export function describeAcquisition(account: LeaveAccount): string {
	return account.acquisition.kind === 'monthly'
		? `${formatDaysCompact(Math.round(account.acquisition.perMonth * 100) / 100)} j par mois`
		: `${formatDaysCompact(account.acquisition.days)} j par an`
}
