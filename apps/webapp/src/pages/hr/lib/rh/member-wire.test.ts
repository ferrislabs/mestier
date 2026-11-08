import { expect, test } from 'vitest'
import { weekly } from '#/pages/hr/lib/rh/contract-schedule'
import {
	CONTRACT_TYPE_FROM_WIRE,
	CONTRACT_TYPE_TO_WIRE,
	type ContractRecord,
	contractFailureOf,
	contractOf,
	contractRequestOf,
	DOCUMENT_KIND_FROM_WIRE,
	DOCUMENT_KIND_TO_WIRE,
	type DocumentRecord,
	documentOf,
	documentRequestOf,
	overlayContracts,
	overlayDocuments,
	scheduleOf,
	scheduleRequestOf,
} from '#/pages/hr/lib/rh/member-wire'
import type {
	ContractDraft,
	ContractSchedule,
	ContractType,
	DocumentKind,
	RhState,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

const DAY: WorkDay = {
	start: '08:00',
	end: '17:00',
	lunch: { start: '12:00', end: '13:00' },
}
const SHORT: WorkDay = { start: '09:00', end: '12:00', lunch: null }

const WEEKLY: ContractSchedule = weekly([DAY, DAY, DAY, DAY, SHORT, null, null])

const ROTATION: ContractSchedule = {
	kind: 'rotation',
	anchor: '2026-01-05',
	blocks: [
		{
			length: 2,
			unit: 'weeks',
			label: 'Chantier',
			week: [DAY, DAY, DAY, DAY, DAY, null, null],
		},
		{
			length: 1,
			unit: 'months',
			label: '',
			week: [null, null, null, null, null, null, null],
		},
	],
}

const VARIABLE: ContractSchedule = { kind: 'variable', defaultDay: DAY }

test('spec_rh_2_6 every contract type round-trips through its explicit mapping', () => {
	const types = Object.keys(CONTRACT_TYPE_TO_WIRE) as ContractType[]
	expect(types).toHaveLength(5)
	for (const type of types)
		expect(CONTRACT_TYPE_FROM_WIRE[CONTRACT_TYPE_TO_WIRE[type]]).toBe(type)
	expect(CONTRACT_TYPE_TO_WIRE).toEqual({
		cdi: 'CDI',
		cdd: 'CDD',
		apprentissage: 'APPRENTICESHIP',
		interim: 'INTERIM',
		gerant: 'MANAGER',
	})
})

test('spec_rh_2_6 every document kind round-trips through its explicit mapping', () => {
	const kinds = Object.keys(DOCUMENT_KIND_TO_WIRE) as DocumentKind[]
	expect(kinds).toHaveLength(5)
	for (const kind of kinds)
		expect(DOCUMENT_KIND_FROM_WIRE[DOCUMENT_KIND_TO_WIRE[kind]]).toBe(kind)
	expect(DOCUMENT_KIND_TO_WIRE.diploma).toBe('DIPLOMA')
})

test('spec_rh_2_6 the three schedule kinds round-trip', () => {
	for (const schedule of [WEEKLY, ROTATION, VARIABLE])
		expect(scheduleOf(scheduleRequestOf(schedule))).toEqual(schedule)
})

test('spec_rh_2_6 the schedule goes out in snake_case with a null lunch kept explicit', () => {
	expect(scheduleRequestOf(VARIABLE)).toEqual({
		kind: 'variable',
		default_day: {
			start: '08:00',
			end: '17:00',
			lunch: { start: '12:00', end: '13:00' },
		},
	})
	const week = scheduleRequestOf(WEEKLY)
	expect(week.kind === 'weekly' && week.week[4]).toEqual({
		start: '09:00',
		end: '12:00',
		lunch: null,
	})
	expect(week.kind === 'weekly' && week.week[6]).toBeNull()
})

test('spec_rh_2_6 a contract draft round-trips and keeps open ends as null', () => {
	const draft: ContractDraft = {
		type: 'apprentissage',
		startDate: '2026-09-01',
		endDate: null,
		trialEndDate: '2026-10-31',
		schedule: ROTATION,
	}
	const request = contractRequestOf(draft)
	expect(request).toMatchObject({
		type: 'APPRENTICESHIP',
		start_date: '2026-09-01',
		end_date: null,
		trial_end_date: '2026-10-31',
	})
	const record: ContractRecord = {
		...request,
		id: 'c-1',
		member_id: 'm-1',
		organization_id: 'org-1',
	}
	expect(contractOf(record)).toEqual({ ...draft, id: 'c-1' })
	expect(
		contractOf({ ...record, end_date: undefined, trial_end_date: undefined }),
	).toMatchObject({ endDate: null, trialEndDate: null })
})

test('spec_rh_2_6 a document keeps its file key and the day it was added', () => {
	const record: DocumentRecord = {
		id: 'd-1',
		organization_id: 'org-1',
		member_id: 'm-1',
		name: 'CNI.pdf',
		kind: 'IDENTITY',
		file_key: 'uploads/cni',
		added_at: '2026-11-05T22:30:00Z',
	}
	expect(documentOf(record)).toEqual({
		id: 'd-1',
		name: 'CNI.pdf',
		kind: 'identity',
		addedAt: '2026-11-05',
		fileKey: 'uploads/cni',
	})
	expect(documentRequestOf(' CNI.pdf ', 'identity', 'uploads/cni')).toEqual({
		name: 'CNI.pdf',
		kind: 'IDENTITY',
		file_key: 'uploads/cni',
	})
})

function stateOf(ids: string[]): RhState {
	return {
		members: ids.map((id) => ({
			id,
			contracts: [{ id: 'stale' }],
			documents: [{ id: 'stale' }],
		})),
	} as unknown as RhState
}

test('spec_rh_2_6 the overlay replaces contracts and documents per member, sorted, empty without rows', () => {
	const request = contractRequestOf({
		type: 'cdi',
		startDate: '2026-01-01',
		endDate: null,
		trialEndDate: null,
		schedule: WEEKLY,
	})
	const row = (
		id: string,
		memberId: string,
		start: string,
	): ContractRecord => ({
		...request,
		id,
		member_id: memberId,
		organization_id: 'org-1',
		start_date: start,
	})
	const contracts = overlayContracts(stateOf(['a', 'b']), [
		row('c-2', 'a', '2027-01-01'),
		row('c-1', 'a', '2026-01-01'),
	])
	expect(contracts.members[0]?.contracts.map((c) => c.id)).toEqual([
		'c-1',
		'c-2',
	])
	expect(contracts.members[1]?.contracts).toEqual([])

	const documents = overlayDocuments(stateOf(['a', 'b']), [
		{
			id: 'd-1',
			organization_id: 'org-1',
			member_id: 'b',
			name: 'x',
			kind: 'OTHER',
			file_key: 'k',
			added_at: '2026-02-01T00:00:00Z',
		},
	])
	expect(documents.members[0]?.documents).toEqual([])
	expect(documents.members[1]?.documents.map((d) => d.id)).toEqual(['d-1'])
})

const refusal = (field: string, code = 'E_UNPROCESSABLE_ENTITY') =>
	Object.assign(new Error('refused'), {
		status: 422,
		data: { code, message: 'refused', details: { field } },
	})

test('spec_rh_2_6 a refusal names the field or the day of the schedule', () => {
	expect(contractFailureOf(refusal('trial_end_date'))).toMatchObject({
		field: 'trialEndDate',
		overlap: false,
	})
	expect(contractFailureOf(refusal('end_date'))).toMatchObject({
		field: 'endDate',
	})
	expect(contractFailureOf(refusal('schedule.week[2]'))).toEqual({
		field: 'schedule',
		overlap: false,
		message: 'Rythme, mercredi : refused',
	})
	expect(contractFailureOf(refusal('schedule.blocks[1].week[0]')).message).toBe(
		'Séquence 2, lundi : refused',
	)
	expect(contractFailureOf(refusal('schedule.blocks[0].length')).message).toBe(
		'Séquence 1 : refused',
	)
	expect(contractFailureOf(refusal('schedule.default_day')).message).toBe(
		'Horaires par défaut : refused',
	)
})

test('spec_rh_2_6 an overlap, 422 or 409, lands on the start date', () => {
	expect(
		contractFailureOf(refusal('start_date', 'E_MEMBER_CONTRACT_OVERLAP')),
	).toMatchObject({ field: 'startDate', overlap: true })
	expect(
		contractFailureOf(Object.assign(new Error('conflict'), { status: 409 })),
	).toMatchObject({ field: 'startDate', overlap: true })
	expect(contractFailureOf(new Error('boom'))).toEqual({
		field: null,
		overlap: false,
		message: 'boom',
	})
})
