import type { Schemas } from '#/api/api.client'
import { mutationErrorMessage } from '#/lib/api-error'
import type {
	ExtraDraft,
	ExtraPeriod,
	ExtraStatus,
	RhState,
	Surcharge,
	SurchargeDraft,
	SurchargeKind,
} from '#/pages/hr/lib/rh/types'

export type SurchargeRecord = Schemas.SurchargeResponse
export type SurchargeRequest = Schemas.SurchargeRequest
export type ExtraPeriodRecord = Schemas.ExtraPeriodResponse
export type ExtraPeriodRequest = Schemas.CreateExtraPeriodRequest

export const SURCHARGE_KIND_TO_WIRE: Record<
	SurchargeKind,
	Schemas.SurchargeKind
> = {
	overtime: 'OVERTIME',
	sunday: 'SUNDAY',
	holiday: 'HOLIDAY',
	night: 'NIGHT',
	other: 'OTHER',
}

export const SURCHARGE_KIND_FROM_WIRE: Record<
	Schemas.SurchargeKind,
	SurchargeKind
> = {
	OVERTIME: 'overtime',
	SUNDAY: 'sunday',
	HOLIDAY: 'holiday',
	NIGHT: 'night',
	OTHER: 'other',
}

export const EXTRA_STATUS_TO_WIRE: Record<ExtraStatus, Schemas.ExtraStatus> = {
	pending: 'PENDING',
	approved: 'APPROVED',
	rejected: 'REJECTED',
	cancelled: 'CANCELLED',
}

export const EXTRA_STATUS_FROM_WIRE: Record<Schemas.ExtraStatus, ExtraStatus> =
	{
		PENDING: 'pending',
		APPROVED: 'approved',
		REJECTED: 'rejected',
		CANCELLED: 'cancelled',
	}

export function rateOf(basisPoints: number): number {
	return basisPoints / 100
}

export function basisPointsOf(rate: number): number {
	return Math.round(rate * 100)
}

export function surchargeOf(record: SurchargeRecord): Surcharge {
	return {
		id: record.id,
		name: record.name,
		rate: rateOf(record.rate_bp),
		kind: SURCHARGE_KIND_FROM_WIRE[record.kind],
		archived: Boolean(record.archived_at),
	}
}

export function surchargeRequestOf(draft: SurchargeDraft): SurchargeRequest {
	return {
		name: draft.name.trim(),
		rate_bp: basisPointsOf(draft.rate),
		kind: SURCHARGE_KIND_TO_WIRE[draft.kind],
	}
}

export function extraOf(record: ExtraPeriodRecord): ExtraPeriod {
	return {
		id: record.id,
		memberId: record.member_id,
		date: record.date,
		start: record.start,
		end: record.end,
		lunch: record.lunch
			? { start: record.lunch.start, end: record.lunch.end }
			: null,
		reason: record.reason,
		surchargeId: record.surcharge_id ?? null,
		rate: rateOf(record.rate_bp),
		status: EXTRA_STATUS_FROM_WIRE[record.status],
		createdAt: record.created_at,
		createdById: record.created_by_member_id ?? '',
		decidedById: record.decided_by_member_id ?? null,
		decidedAt: record.decided_at ?? null,
		decisionComment: record.decision_comment ?? '',
	}
}

export function extraRequestOf(
	draft: ExtraDraft,
	approve: boolean,
): ExtraPeriodRequest {
	return {
		member_id: draft.memberId,
		date: draft.date,
		start: draft.start,
		end: draft.end,
		lunch: draft.lunch ?? undefined,
		reason: draft.reason.trim(),
		surcharge_id: draft.surchargeId ?? undefined,
		rate_bp: draft.surchargeId === null ? basisPointsOf(draft.rate) : undefined,
		approve,
	}
}

export function overlaySurcharges(
	state: RhState,
	records: readonly SurchargeRecord[],
): RhState {
	return { ...state, surcharges: records.map(surchargeOf) }
}

export function overlayExtras(
	state: RhState,
	records: readonly ExtraPeriodRecord[],
): RhState {
	return { ...state, extras: records.map(extraOf) }
}

export type SurchargeFailureField = 'name' | 'rate'

export type SurchargeFailure = {
	field: SurchargeFailureField | null
	message: string
}

export type ExtraFailureField =
	| 'member'
	| 'start'
	| 'end'
	| 'lunch'
	| 'reason'
	| 'rate'
	| 'surcharge'
	| 'comment'
	| 'status'

export type ExtraFailure = {
	field: ExtraFailureField | null
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

const SURCHARGE_FIELDS: Record<string, SurchargeFailureField> = {
	name: 'name',
	rate_bp: 'rate',
}

const SURCHARGE_MESSAGES: Record<SurchargeFailureField, string> = {
	name: 'Donne un nom à cette majoration.',
	rate: 'Le taux doit être compris entre 0 et 200 %.',
}

export function surchargeFailureOf(error: unknown): SurchargeFailure {
	const failure = failureOf(error)
	if (failure.status === 404)
		return { field: null, message: 'Cette majoration est introuvable.' }
	const field = SURCHARGE_FIELDS[failure.data?.details?.field ?? '']
	if (failure.status === 422 && field)
		return { field, message: SURCHARGE_MESSAGES[field] }
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}

const EXTRA_FIELDS: Record<string, ExtraFailureField> = {
	member_id: 'member',
	start: 'start',
	end: 'end',
	lunch: 'lunch',
	reason: 'reason',
	rate_bp: 'rate',
	surcharge_id: 'surcharge',
	comment: 'comment',
	status: 'status',
}

const EXTRA_MESSAGES: Record<ExtraFailureField, string> = {
	member: 'Ce membre est introuvable.',
	start: 'L’heure de début n’est pas valide.',
	end: 'La fin doit être après le début.',
	lunch: 'La pause doit tenir entre le début et la fin.',
	reason: 'Indique la raison de ces heures.',
	rate: 'La majoration doit être comprise entre 0 et 200 %.',
	surcharge: 'Cette majoration est archivée ou introuvable.',
	comment: 'Un refus doit être motivé par un commentaire.',
	status: 'Cette déclaration a déjà été traitée.',
}

export function extraFailureOf(error: unknown): ExtraFailure {
	const failure = failureOf(error)
	if (failure.status === 404)
		return { field: null, message: 'Cette déclaration est introuvable.' }
	const field = EXTRA_FIELDS[failure.data?.details?.field ?? '']
	if (failure.status === 422 && field)
		return { field, message: EXTRA_MESSAGES[field] }
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}
