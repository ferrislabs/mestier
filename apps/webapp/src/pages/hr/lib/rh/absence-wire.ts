import type { Schemas } from '#/api/api.client'
import { mutationErrorMessage } from '#/lib/api-error'
import type {
	Absence,
	AbsenceDraft,
	AbsenceKind,
	AbsenceStatus,
	AbsenceType,
	AbsenceTypeDraft,
	CountMode,
	DayPart,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type AbsenceTypeRecord = Schemas.AbsenceTypeResponse
export type AbsenceTypeRequest = Schemas.AbsenceTypeRequest
export type AbsenceRecord = Schemas.AbsenceResponse
export type AbsenceRequest = Schemas.CreateAbsenceRequest

export const COUNT_MODE_TO_WIRE: Record<CountMode, Schemas.CountMode> = {
	working: 'WORKING',
	workable: 'WORKABLE',
	calendar: 'CALENDAR',
}

export const COUNT_MODE_FROM_WIRE: Record<Schemas.CountMode, CountMode> = {
	WORKING: 'working',
	WORKABLE: 'workable',
	CALENDAR: 'calendar',
}

export const ABSENCE_KIND_TO_WIRE: Record<AbsenceKind, Schemas.AbsenceKind> = {
	leave: 'LEAVE',
	sick: 'SICK',
	unavailable: 'UNAVAILABLE',
}

export const ABSENCE_KIND_FROM_WIRE: Record<Schemas.AbsenceKind, AbsenceKind> =
	{
		LEAVE: 'leave',
		SICK: 'sick',
		UNAVAILABLE: 'unavailable',
	}

export const DAY_PART_TO_WIRE: Record<DayPart, Schemas.DayPart> = {
	morning: 'MORNING',
	afternoon: 'AFTERNOON',
}

export const DAY_PART_FROM_WIRE: Record<Schemas.DayPart, DayPart> = {
	MORNING: 'morning',
	AFTERNOON: 'afternoon',
}

export const ABSENCE_STATUS_FROM_WIRE: Record<
	Schemas.AbsenceStatus,
	AbsenceStatus
> = {
	PENDING: 'pending',
	APPROVED: 'approved',
	REJECTED: 'rejected',
	CANCELLED: 'cancelled',
}

export function absenceTypeOf(record: AbsenceTypeRecord): AbsenceType {
	return {
		id: record.id,
		name: record.name,
		hue: record.hue,
		kind: ABSENCE_KIND_FROM_WIRE[record.kind],
		countMode: COUNT_MODE_FROM_WIRE[record.count_mode],
		accountId: record.account_id ?? null,
		paid: record.paid,
		needsApproval: record.needs_approval,
		needsProof: record.needs_proof,
		archived: Boolean(record.archived_at),
	}
}

export function absenceTypeRequestOf(
	draft: AbsenceTypeDraft,
): AbsenceTypeRequest {
	return {
		name: draft.name.trim(),
		hue: draft.hue,
		kind: ABSENCE_KIND_TO_WIRE[draft.kind],
		count_mode: COUNT_MODE_TO_WIRE[draft.countMode],
		account_id: draft.accountId,
		paid: draft.paid,
		needs_approval: draft.needsApproval,
		needs_proof: draft.needsProof,
	}
}

export function absenceOf(record: AbsenceRecord): Absence {
	return {
		id: record.id,
		memberId: record.member_id,
		typeId: record.type_id,
		startDate: record.start_date,
		startPart: DAY_PART_FROM_WIRE[record.start_part],
		endDate: record.end_date,
		endPart: DAY_PART_FROM_WIRE[record.end_part],
		comment: record.note ?? '',
		proofKey: record.proof_file_key ?? null,
		status: ABSENCE_STATUS_FROM_WIRE[record.status],
		createdAt: record.created_at,
		createdById: record.created_by_member_id ?? '',
		decidedById: record.decided_by_member_id ?? null,
		decidedAt: record.decided_at ?? null,
		decisionComment: record.decision_comment ?? '',
	}
}

export function absenceRequestOf(
	draft: AbsenceDraft,
	proofKey: string | null,
): AbsenceRequest {
	const note = draft.comment.trim()
	return {
		member_id: draft.memberId,
		type_id: draft.typeId,
		start_date: draft.startDate,
		start_part: DAY_PART_TO_WIRE[draft.startPart],
		end_date: draft.endDate,
		end_part: DAY_PART_TO_WIRE[draft.endPart],
		note: note === '' ? undefined : note,
		proof_file_key: proofKey ?? undefined,
	}
}

export function overlayAbsenceTypes(
	state: RhState,
	records: readonly AbsenceTypeRecord[],
): RhState {
	return { ...state, absenceTypes: records.map(absenceTypeOf) }
}

export function overlayAbsences(
	state: RhState,
	records: readonly AbsenceRecord[],
): RhState {
	return {
		...state,
		absences: records
			.map(absenceOf)
			.sort((left, right) => right.createdAt.localeCompare(left.createdAt)),
	}
}

export type TypeFailureField = 'name' | 'hue'

export type TypeFailure = { field: TypeFailureField | null; message: string }

export type AbsenceFailureField =
	| 'type'
	| 'startDate'
	| 'endDate'
	| 'comment'
	| 'proof'
	| 'status'

export type AbsenceFailure = {
	field: AbsenceFailureField | null
	message: string
}

type ApiFailure = {
	status?: number
	message?: string
	data?: { message?: string; details?: { field?: string } }
}

function failureOf(error: unknown): ApiFailure {
	return (
		typeof error === 'object' && error !== null ? error : {}
	) as ApiFailure
}

const TYPE_MESSAGES: Record<TypeFailureField, string> = {
	name: 'Le nom ne peut pas être vide.',
	hue: 'La teinte doit être comprise entre 0 et 360.',
}

export function typeFailureOf(error: unknown): TypeFailure {
	const failure = failureOf(error)
	const path = failure.data?.details?.field
	if (path === 'name' || path === 'hue')
		return { field: path, message: TYPE_MESSAGES[path] }
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}

const ABSENCE_FIELDS: Record<string, AbsenceFailureField> = {
	type_id: 'type',
	start_date: 'startDate',
	end_date: 'endDate',
	note: 'comment',
	comment: 'comment',
	proof_file_key: 'proof',
	status: 'status',
}

const ABSENCE_MESSAGES: Record<AbsenceFailureField, string> = {
	type: 'Ce motif est archivé ou introuvable.',
	startDate: 'La date de début n’est pas valide.',
	endDate: 'La fin de l’absence précède son début.',
	comment: 'Le commentaire ne peut pas être vide.',
	proof: 'Le justificatif n’est pas valide.',
	status: 'Cette demande a déjà été traitée.',
}

export function absenceFailureOf(error: unknown): AbsenceFailure {
	const failure = failureOf(error)
	if (failure.status === 404)
		return { field: null, message: 'Cette demande est introuvable.' }
	const path = failure.data?.details?.field ?? ''
	const field = ABSENCE_FIELDS[path]
	if (failure.status === 422 && field) {
		if (field === 'proof' && /needs a proof/.test(failure.data?.message ?? ''))
			return {
				field,
				message: 'Ce motif exige un justificatif avant la validation.',
			}
		if (path === 'comment')
			return {
				field,
				message: 'Un refus doit être motivé par un commentaire.',
			}
		return { field, message: ABSENCE_MESSAGES[field] }
	}
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}
