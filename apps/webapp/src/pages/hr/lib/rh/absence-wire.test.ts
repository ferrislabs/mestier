import { expect, test } from 'vitest'
import {
	ABSENCE_KIND_FROM_WIRE,
	ABSENCE_KIND_TO_WIRE,
	ABSENCE_STATUS_FROM_WIRE,
	type AbsenceRecord,
	type AbsenceTypeRecord,
	absenceFailureOf,
	absenceOf,
	absenceRequestOf,
	absenceTypeOf,
	absenceTypeRequestOf,
	COUNT_MODE_FROM_WIRE,
	COUNT_MODE_TO_WIRE,
	DAY_PART_FROM_WIRE,
	DAY_PART_TO_WIRE,
	overlayAbsences,
	overlayAbsenceTypes,
	typeFailureOf,
} from '#/pages/hr/lib/rh/absence-wire'
import type {
	AbsenceDraft,
	AbsenceKind,
	AbsenceTypeDraft,
	CountMode,
	DayPart,
	RhState,
} from '#/pages/hr/lib/rh/types'

const TYPE_ID = '00000000-0000-4000-8000-000000000001'
const ACCOUNT_ID = '00000000-0000-4000-8000-0000000000aa'

function typeRecord(patch: Partial<AbsenceTypeRecord> = {}): AbsenceTypeRecord {
	return {
		id: TYPE_ID,
		organization_id: 'org-1',
		name: 'Congés payés',
		hue: 155,
		count_mode: 'WORKING',
		account_id: null,
		paid: true,
		needs_approval: true,
		needs_proof: false,
		kind: 'LEAVE',
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...patch,
	}
}

function absenceRecord(patch: Partial<AbsenceRecord> = {}): AbsenceRecord {
	return {
		id: 'a-1',
		organization_id: 'org-1',
		member_id: 'm-1',
		type_id: TYPE_ID,
		kind: 'LEAVE',
		start_date: '2026-07-06',
		start_part: 'AFTERNOON',
		end_date: '2026-07-08',
		end_part: 'MORNING',
		starts_at: '2026-07-06T12:00:00Z',
		ends_at: '2026-07-08T12:00:00Z',
		all_day: false,
		note: 'Vacances',
		proof_file_key: 'uploads/proof.pdf',
		status: 'PENDING',
		created_by_member_id: 'm-2',
		decided_by_member_id: null,
		decided_at: null,
		decision_comment: null,
		created_at: '2026-06-01T08:00:00Z',
		updated_at: '2026-06-01T08:00:00Z',
		...patch,
	}
}

const EMPTY_STATE = {
	members: [],
	absenceTypes: [],
	accounts: [],
	absences: [],
	adjustments: [],
	overrides: [],
	surcharges: [],
	extras: [],
} satisfies RhState

test('count modes round-trip through the explicit tables', () => {
	for (const mode of ['working', 'workable', 'calendar'] as CountMode[])
		expect(COUNT_MODE_FROM_WIRE[COUNT_MODE_TO_WIRE[mode]]).toBe(mode)
	expect(COUNT_MODE_TO_WIRE.workable).toBe('WORKABLE')
})

test('kinds, parts and statuses round-trip through the explicit tables', () => {
	for (const kind of ['leave', 'sick', 'unavailable'] as AbsenceKind[])
		expect(ABSENCE_KIND_FROM_WIRE[ABSENCE_KIND_TO_WIRE[kind]]).toBe(kind)
	for (const part of ['morning', 'afternoon'] as DayPart[])
		expect(DAY_PART_FROM_WIRE[DAY_PART_TO_WIRE[part]]).toBe(part)
	expect(ABSENCE_STATUS_FROM_WIRE).toEqual({
		PENDING: 'pending',
		APPROVED: 'approved',
		REJECTED: 'rejected',
		CANCELLED: 'cancelled',
	})
})

test('a type record maps to the model and back to the same request', () => {
	const type = absenceTypeOf(typeRecord({ count_mode: 'CALENDAR' }))
	expect(type).toEqual({
		id: TYPE_ID,
		name: 'Congés payés',
		hue: 155,
		kind: 'leave',
		countMode: 'calendar',
		accountId: null,
		paid: true,
		needsApproval: true,
		needsProof: false,
		archived: false,
	})
	const { id: _id, archived: _archived, ...draft } = type
	expect(absenceTypeRequestOf(draft)).toEqual({
		name: 'Congés payés',
		hue: 155,
		kind: 'LEAVE',
		count_mode: 'CALENDAR',
		account_id: null,
		paid: true,
		needs_approval: true,
		needs_proof: false,
	})
})

test('an archived record is archived and a UUID account id is kept', () => {
	const type = absenceTypeOf(
		typeRecord({ archived_at: '2026-02-01T00:00:00Z', account_id: ACCOUNT_ID }),
	)
	expect(type.archived).toBe(true)
	expect(type.accountId).toBe(ACCOUNT_ID)
})

test('the request carries the account id as the API knows it', () => {
	const draft: AbsenceTypeDraft = {
		name: '  Déménagement ',
		hue: 10,
		kind: 'unavailable',
		countMode: 'working',
		accountId: ACCOUNT_ID,
		paid: false,
		needsApproval: false,
		needsProof: true,
	}
	expect(absenceTypeRequestOf(draft).account_id).toBe(ACCOUNT_ID)
	expect(absenceTypeRequestOf(draft).name).toBe('Déménagement')
	expect(
		absenceTypeRequestOf({ ...draft, accountId: null }).account_id,
	).toBeNull()
})

test('an absence record maps to the model', () => {
	expect(absenceOf(absenceRecord())).toEqual({
		id: 'a-1',
		memberId: 'm-1',
		typeId: TYPE_ID,
		startDate: '2026-07-06',
		startPart: 'afternoon',
		endDate: '2026-07-08',
		endPart: 'morning',
		comment: 'Vacances',
		proofKey: 'uploads/proof.pdf',
		status: 'pending',
		createdAt: '2026-06-01T08:00:00Z',
		createdById: 'm-2',
		decidedById: null,
		decidedAt: null,
		decisionComment: '',
	})
})

test('a decided absence keeps its decider, date and comment', () => {
	const absence = absenceOf(
		absenceRecord({
			status: 'REJECTED',
			decided_by_member_id: 'm-3',
			decided_at: '2026-06-02T09:00:00Z',
			decision_comment: 'Trop chargé',
			note: null,
			proof_file_key: null,
			created_by_member_id: null,
		}),
	)
	expect(absence).toMatchObject({
		status: 'rejected',
		decidedById: 'm-3',
		decidedAt: '2026-06-02T09:00:00Z',
		decisionComment: 'Trop chargé',
		comment: '',
		proofKey: null,
		createdById: '',
	})
})

test('a draft becomes a request in the new shape', () => {
	const draft: AbsenceDraft = {
		memberId: 'm-1',
		typeId: TYPE_ID,
		startDate: '2026-07-06',
		startPart: 'afternoon',
		endDate: '2026-07-08',
		endPart: 'morning',
		comment: '  Vacances ',
		proofName: 'proof.pdf',
	}
	expect(absenceRequestOf(draft, 'uploads/proof.pdf')).toEqual({
		member_id: 'm-1',
		type_id: TYPE_ID,
		start_date: '2026-07-06',
		start_part: 'AFTERNOON',
		end_date: '2026-07-08',
		end_part: 'MORNING',
		note: 'Vacances',
		proof_file_key: 'uploads/proof.pdf',
	})
	const bare = absenceRequestOf({ ...draft, comment: '   ' }, null)
	expect(bare.note).toBeUndefined()
	expect(bare.proof_file_key).toBeUndefined()
})

test('the overlay replaces the types and keeps the account the API gives', () => {
	const state = overlayAbsenceTypes(
		{
			...EMPTY_STATE,
			absenceTypes: [{ ...absenceTypeOf(typeRecord()), id: 'old' }],
		},
		[
			typeRecord({ account_id: ACCOUNT_ID }),
			typeRecord({ id: 'other', name: 'RTT' }),
		],
	)
	expect(state.absenceTypes.map((type) => type.id)).toEqual([TYPE_ID, 'other'])
	expect(state.absenceTypes[0]?.accountId).toBe(ACCOUNT_ID)
	expect(state.absenceTypes[1]?.accountId).toBeNull()
})

test('the overlay replaces the absences, newest first', () => {
	const stale = {
		...absenceOf(absenceRecord({ id: 'stale' })),
	}
	const state = overlayAbsences({ ...EMPTY_STATE, absences: [stale] }, [
		absenceRecord({ id: 'old', created_at: '2026-01-01T00:00:00Z' }),
		absenceRecord({ id: 'new', created_at: '2026-05-01T00:00:00Z' }),
	])
	expect(state.absences.map((absence) => absence.id)).toEqual(['new', 'old'])
})

test('a type refusal is attached to its field in French', () => {
	expect(
		typeFailureOf({
			status: 422,
			message: 'refused',
			data: { message: 'name: must not be empty', details: { field: 'name' } },
		}),
	).toEqual({ field: 'name', message: 'Le nom ne peut pas être vide.' })
	expect(
		typeFailureOf({ status: 422, data: { details: { field: 'hue' } } }).field,
	).toBe('hue')
	expect(typeFailureOf(new Error('boom'))).toEqual({
		field: null,
		message: 'boom',
	})
})

test('an absence refusal is attached to its field in French', () => {
	const refuse = (field: string, message = '') =>
		absenceFailureOf({
			status: 422,
			message: 'refused',
			data: { message, details: { field } },
		})
	expect(refuse('end_date').field).toBe('endDate')
	expect(refuse('type_id').field).toBe('type')
	expect(refuse('note').field).toBe('comment')
	expect(refuse('comment').message).toBe(
		'Un refus doit être motivé par un commentaire.',
	)
	expect(refuse('status').message).toBe('Cette demande a déjà été traitée.')
	const proof = refuse(
		'proof_file_key',
		'proof_file_key: this absence type needs a proof before approval',
	)
	expect(proof.field).toBe('proof')
	expect(proof.message).toBe(
		'Ce motif exige un justificatif avant la validation.',
	)
	expect(refuse('proof_file_key', 'not a valid storage key').message).toBe(
		'Le justificatif n’est pas valide.',
	)
})

test('a 403 and a 404 get their own messages', () => {
	expect(absenceFailureOf({ status: 403 }).message).toBe(
		"Vous n'avez plus la permission nécessaire pour cette action.",
	)
	expect(absenceFailureOf({ status: 404 }).message).toBe(
		'Cette demande est introuvable.',
	)
	expect(absenceFailureOf({ status: 403 }).field).toBeNull()
})
