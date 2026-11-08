import type { Schemas } from '#/api/api.client'
import { mutationErrorMessage } from '#/lib/api-error'
import type { AccountSettingsDraft } from '#/pages/hr/lib/rh/rights-model'
import type {
	AdjustmentDraft,
	AdjustmentKind,
	BalanceAdjustment,
	LeaveAccount,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type LeaveAccountRecord = Schemas.LeaveAccountResponse
export type LeaveAccountRequest = Schemas.LeaveAccountRequest
export type LeaveAdjustmentRecord = Schemas.LeaveAdjustmentResponse
export type LeaveAdjustmentRequest = Schemas.CreateLeaveAdjustmentRequest

export const ADJUSTMENT_KIND_TO_WIRE: Record<
	AdjustmentKind,
	Schemas.AdjustmentKind
> = {
	adjustment: 'ADJUSTMENT',
	carry_over: 'CARRY_OVER',
}

export const ADJUSTMENT_KIND_FROM_WIRE: Record<
	Schemas.AdjustmentKind,
	AdjustmentKind
> = {
	ADJUSTMENT: 'adjustment',
	CARRY_OVER: 'carry_over',
}

export const CARRY_OVER_FALLBACK_REASON = 'Report'

export function accountOf(record: LeaveAccountRecord): LeaveAccount {
	return {
		id: record.id,
		name: record.name,
		acquisition:
			record.acquisition.kind === 'MONTHLY'
				? { kind: 'monthly', perMonth: record.acquisition.per_month }
				: { kind: 'annual', days: record.acquisition.days },
		periodStartMonth: record.period_start_month,
		carryOverMax: record.carry_over_max,
		archived: Boolean(record.archived_at),
	}
}

export function accountRequestOf(
	account: LeaveAccount,
	settings: AccountSettingsDraft,
): LeaveAccountRequest {
	return {
		name: account.name,
		acquisition:
			settings.kind === 'monthly'
				? { kind: 'MONTHLY', per_month: settings.value }
				: { kind: 'ANNUAL', days: settings.value },
		period_start_month: settings.periodStartMonth,
		carry_over_max: settings.carryOverMax,
	}
}

export function adjustmentOf(record: LeaveAdjustmentRecord): BalanceAdjustment {
	return {
		id: record.id,
		memberId: record.member_id,
		accountId: record.account_id,
		date: record.date,
		days: record.days,
		kind: ADJUSTMENT_KIND_FROM_WIRE[record.kind],
		reason: record.reason,
		authorId: record.author_member_id ?? '',
	}
}

export function adjustmentRequestOf(
	draft: AdjustmentDraft,
): LeaveAdjustmentRequest {
	const reason = draft.reason.trim()
	return {
		member_id: draft.memberId,
		account_id: draft.accountId,
		date: draft.date,
		days: draft.days,
		kind: ADJUSTMENT_KIND_TO_WIRE[draft.kind],
		reason:
			reason === '' && draft.kind === 'carry_over'
				? CARRY_OVER_FALLBACK_REASON
				: reason,
	}
}

export function overlayAccounts(
	state: RhState,
	records: readonly LeaveAccountRecord[],
): RhState {
	return { ...state, accounts: records.map(accountOf) }
}

export function overlayAdjustments(
	state: RhState,
	records: readonly LeaveAdjustmentRecord[],
): RhState {
	return { ...state, adjustments: records.map(adjustmentOf) }
}

export type AccountFailureField = 'value' | 'periodStartMonth' | 'carryOverMax'

export type AccountFailure = {
	field: AccountFailureField | null
	message: string
}

export type AdjustmentFailureField = 'days' | 'reason' | 'account' | 'member'

export type AdjustmentFailure = {
	field: AdjustmentFailureField | null
	message: string
}

type ApiFailure = {
	status?: number
	data?: { message?: string; details?: { field?: string } }
}

function failureOf(error: unknown): ApiFailure {
	return (
		typeof error === 'object' && error !== null ? error : {}
	) as ApiFailure
}

const ACCOUNT_FIELDS: Record<string, AccountFailureField> = {
	acquisition: 'value',
	period_start_month: 'periodStartMonth',
	carry_over_max: 'carryOverMax',
}

const ACCOUNT_MESSAGES: Record<AccountFailureField, string> = {
	value: 'L’acquisition doit être positive, avec deux décimales au plus.',
	periodStartMonth: 'Choisis un mois de début de période.',
	carryOverMax:
		'Le report maximal doit être positif ou nul, avec deux décimales au plus.',
}

export function accountFailureOf(error: unknown): AccountFailure {
	const failure = failureOf(error)
	if (failure.status === 404)
		return { field: null, message: 'Ce compte est introuvable.' }
	const field = ACCOUNT_FIELDS[failure.data?.details?.field ?? '']
	if (failure.status === 422 && field)
		return { field, message: ACCOUNT_MESSAGES[field] }
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}

const ADJUSTMENT_FIELDS: Record<string, AdjustmentFailureField> = {
	days: 'days',
	reason: 'reason',
	account_id: 'account',
	member_id: 'member',
}

export function adjustmentFailureOf(error: unknown): AdjustmentFailure {
	const failure = failureOf(error)
	if (failure.status === 404)
		return { field: null, message: 'Ce mouvement est introuvable.' }
	const path = failure.data?.details?.field ?? ''
	const field = ADJUSTMENT_FIELDS[path]
	if (failure.status === 422 && field) {
		const message = failure.data?.message ?? ''
		if (field === 'days' && /exceeds the maximum/.test(message))
			return {
				field,
				message: 'Le report dépasse le maximum autorisé sur ce compte.',
			}
		if (field === 'days' && /always positive/.test(message))
			return { field, message: 'Un report est toujours positif.' }
		if (field === 'days')
			return {
				field,
				message:
					'Indique un nombre de jours différent de zéro, avec deux décimales au plus.',
			}
		if (field === 'reason')
			return { field, message: 'Indique le motif de la régularisation.' }
		if (field === 'account')
			return { field, message: 'Ce compte est archivé ou introuvable.' }
		return { field, message: 'Ce membre est introuvable.' }
	}
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}
