import { expect, test } from 'vitest'
import {
	basisPointsOf,
	EXTRA_STATUS_FROM_WIRE,
	EXTRA_STATUS_TO_WIRE,
	type ExtraPeriodRecord,
	extraFailureOf,
	extraOf,
	extraRequestOf,
	overlayExtras,
	overlaySurcharges,
	rateOf,
	SURCHARGE_KIND_FROM_WIRE,
	SURCHARGE_KIND_TO_WIRE,
	type SurchargeRecord,
	surchargeFailureOf,
	surchargeOf,
	surchargeRequestOf,
} from '#/pages/hr/lib/rh/extra-wire'
import type {
	ExtraDraft,
	ExtraStatus,
	RhState,
	SurchargeKind,
} from '#/pages/hr/lib/rh/types'

const SURCHARGE_ID = '00000000-0000-4000-8000-000000000001'

function surchargeRecord(
	patch: Partial<SurchargeRecord> = {},
): SurchargeRecord {
	return {
		id: SURCHARGE_ID,
		organization_id: 'org-1',
		name: 'Travail du dimanche',
		rate_bp: 10000,
		kind: 'SUNDAY',
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...patch,
	}
}

function extraRecord(
	patch: Partial<ExtraPeriodRecord> = {},
): ExtraPeriodRecord {
	return {
		id: 'e-1',
		organization_id: 'org-1',
		member_id: 'm-1',
		date: '2026-11-14',
		start: '08:00',
		end: '12:00',
		lunch: null,
		reason: 'Rattrapage',
		surcharge_id: SURCHARGE_ID,
		rate_bp: 10000,
		status: 'PENDING',
		created_by_member_id: 'm-2',
		decided_by_member_id: null,
		decided_at: null,
		decision_comment: null,
		created_at: '2026-11-03T17:40:00Z',
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

test('kinds and statuses round-trip through the explicit tables', () => {
	for (const kind of [
		'overtime',
		'sunday',
		'holiday',
		'night',
		'other',
	] as SurchargeKind[])
		expect(SURCHARGE_KIND_FROM_WIRE[SURCHARGE_KIND_TO_WIRE[kind]]).toBe(kind)
	for (const status of [
		'pending',
		'approved',
		'rejected',
		'cancelled',
	] as ExtraStatus[])
		expect(EXTRA_STATUS_FROM_WIRE[EXTRA_STATUS_TO_WIRE[status]]).toBe(status)
	expect(SURCHARGE_KIND_TO_WIRE.overtime).toBe('OVERTIME')
	expect(EXTRA_STATUS_TO_WIRE.cancelled).toBe('CANCELLED')
})

test('a rate converts between percent and basis points without drift', () => {
	expect(rateOf(2500)).toBe(25)
	expect(rateOf(1250)).toBe(12.5)
	expect(basisPointsOf(25)).toBe(2500)
	expect(basisPointsOf(12.5)).toBe(1250)
	expect(basisPointsOf(0.07)).toBe(7)
	for (const bp of [0, 1, 7, 1250, 2500, 10000, 20000])
		expect(basisPointsOf(rateOf(bp))).toBe(bp)
})

test('a surcharge round-trips through the wire', () => {
	const surcharge = surchargeOf(surchargeRecord())
	expect(surcharge).toEqual({
		id: SURCHARGE_ID,
		name: 'Travail du dimanche',
		rate: 100,
		kind: 'sunday',
		archived: false,
	})
	expect(surchargeRequestOf(surcharge)).toEqual({
		name: 'Travail du dimanche',
		rate_bp: 10000,
		kind: 'SUNDAY',
	})
	expect(
		surchargeOf(surchargeRecord({ archived_at: '2026-02-01T00:00:00Z' }))
			.archived,
	).toBe(true)
})

test('a surcharge request trims the name', () => {
	expect(
		surchargeRequestOf({ name: '  Nuit ', rate: 25, kind: 'night' }),
	).toEqual({ name: 'Nuit', rate_bp: 2500, kind: 'NIGHT' })
})

test('an extra period maps to the model with its decision trail', () => {
	expect(extraOf(extraRecord())).toEqual({
		id: 'e-1',
		memberId: 'm-1',
		date: '2026-11-14',
		start: '08:00',
		end: '12:00',
		lunch: null,
		reason: 'Rattrapage',
		surchargeId: SURCHARGE_ID,
		rate: 100,
		status: 'pending',
		createdAt: '2026-11-03T17:40:00Z',
		createdById: 'm-2',
		decidedById: null,
		decidedAt: null,
		decisionComment: '',
	})
	const decided = extraOf(
		extraRecord({
			status: 'REJECTED',
			surcharge_id: null,
			rate_bp: 0,
			lunch: { start: '10:00', end: '10:30' },
			decided_by_member_id: 'm-3',
			decided_at: '2026-11-04T08:00:00Z',
			decision_comment: 'Pas de commande',
		}),
	)
	expect(decided).toMatchObject({
		status: 'rejected',
		surchargeId: null,
		rate: 0,
		lunch: { start: '10:00', end: '10:30' },
		decidedById: 'm-3',
		decisionComment: 'Pas de commande',
	})
})

const DRAFT: ExtraDraft = {
	memberId: 'm-1',
	date: '2026-11-14',
	start: '08:00',
	end: '12:00',
	lunch: null,
	reason: ' Rattrapage ',
	surchargeId: SURCHARGE_ID,
	rate: 100,
}

test('a request with a surcharge sends the surcharge and no free rate', () => {
	expect(extraRequestOf(DRAFT, false)).toEqual({
		member_id: 'm-1',
		date: '2026-11-14',
		start: '08:00',
		end: '12:00',
		lunch: undefined,
		reason: 'Rattrapage',
		surcharge_id: SURCHARGE_ID,
		rate_bp: undefined,
		approve: false,
	})
})

test('a request without a surcharge sends the free rate and the approve flag', () => {
	expect(
		extraRequestOf(
			{
				...DRAFT,
				surchargeId: null,
				rate: 12.5,
				lunch: { start: '10:00', end: '10:30' },
			},
			true,
		),
	).toMatchObject({
		surcharge_id: undefined,
		rate_bp: 1250,
		lunch: { start: '10:00', end: '10:30' },
		approve: true,
	})
})

test('overlays replace the surcharges and the extras of the state', () => {
	const seeded = {
		...EMPTY_STATE,
		surcharges: [
			{
				id: 'seed',
				name: 'Seed',
				rate: 5,
				kind: 'other' as const,
				archived: false,
			},
		],
	}
	expect(overlaySurcharges(seeded, [surchargeRecord()]).surcharges).toEqual([
		surchargeOf(surchargeRecord()),
	])
	expect(overlayExtras(seeded, []).extras).toEqual([])
	expect(overlayExtras(EMPTY_STATE, [extraRecord()]).extras).toEqual([
		extraOf(extraRecord()),
	])
})

test('a refusal names its field and a lost row reads as missing', () => {
	expect(
		extraFailureOf({
			status: 422,
			data: { details: { field: 'end' } },
		}),
	).toEqual({ field: 'end', message: 'La fin doit être après le début.' })
	expect(
		extraFailureOf({
			status: 422,
			data: { details: { field: 'status' } },
		}).field,
	).toBe('status')
	expect(
		extraFailureOf({
			status: 422,
			data: { details: { field: 'comment' } },
		}).field,
	).toBe('comment')
	expect(
		extraFailureOf({
			status: 422,
			data: { details: { field: 'surcharge_id' } },
		}).field,
	).toBe('surcharge')
	expect(extraFailureOf({ status: 404 }).field).toBeNull()
	expect(
		surchargeFailureOf({
			status: 422,
			data: { details: { field: 'rate_bp' } },
		}).field,
	).toBe('rate')
	expect(
		surchargeFailureOf({
			status: 422,
			data: { details: { field: 'name' } },
		}).field,
	).toBe('name')
	expect(surchargeFailureOf({ status: 404 }).message).toBe(
		'Cette majoration est introuvable.',
	)
})
