import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	absenceDays,
	cancelAbsence,
	createAbsence,
	decideAbsence,
	teamConflicts,
	validateAbsence,
} from '#/pages/hr/lib/rh/absences'
import type { AbsenceDraft, ActionContext } from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}

let next = 0
const context: ActionContext = {
	now: '2026-11-04T10:00:00.000Z',
	today: '2026-11-04',
	actorId: 'baptiste',
	newId: () => `id-${++next}`,
}

function draft(patch: Partial<AbsenceDraft> = {}): AbsenceDraft {
	return {
		memberId: 'thomas',
		typeId: 'cp',
		startDate: '2026-12-07',
		startPart: 'morning',
		endDate: '2026-12-09',
		endPart: 'afternoon',
		comment: '',
		proofName: null,
		...patch,
	}
}

test('jours ouvrés : la semaine du 9 au 13 novembre perd le 11 férié', () => {
	assert.equal(
		absenceDays(
			{
				startDate: '2026-11-09',
				startPart: 'morning',
				endDate: '2026-11-13',
				endPart: 'afternoon',
			},
			member('karim'),
			{ countMode: 'working' },
		),
		4,
	)
})

test('jours ouvrés : les jours de repos de la semaine type ne comptent pas', () => {
	const span = {
		startDate: '2026-11-02',
		startPart: 'morning' as const,
		endDate: '2026-11-08',
		endPart: 'afternoon' as const,
	}
	assert.equal(absenceDays(span, member('thomas'), { countMode: 'working' }), 5)
	assert.equal(absenceDays(span, member('lucie'), { countMode: 'working' }), 4)
	assert.equal(
		absenceDays(span, member('thomas'), { countMode: 'workable' }),
		6,
	)
	assert.equal(
		absenceDays(span, member('thomas'), { countMode: 'calendar' }),
		7,
	)
})

test('demi-journées au début et à la fin', () => {
	const span = {
		startDate: '2026-12-07',
		startPart: 'afternoon' as const,
		endDate: '2026-12-09',
		endPart: 'morning' as const,
	}
	assert.equal(absenceDays(span, member('thomas'), { countMode: 'working' }), 2)
	assert.equal(
		absenceDays(
			{ ...span, endDate: '2026-12-07', endPart: 'afternoon' },
			member('thomas'),
			{ countMode: 'working' },
		),
		0.5,
	)
})

test('validateAbsence refuse une fin avant le début', () => {
	assert.equal(
		validateAbsence(
			state,
			draft({ startDate: '2026-12-09', endDate: '2026-12-07' }),
		).length,
		1,
	)
})

test('validateAbsence refuse une période sans jour décompté', () => {
	assert.equal(
		validateAbsence(
			state,
			draft({ startDate: '2026-12-12', endDate: '2026-12-13' }),
		).length,
		1,
	)
})

test('validateAbsence refuse un chevauchement mais accepte l’autre demi-journée', () => {
	const clash = validateAbsence(
		state,
		draft({ startDate: '2026-11-19', endDate: '2026-11-20' }),
	)
	assert.equal(clash.length, 1)
	assert.match(clash[0], /Thomas a déjà une absence/)
	assert.equal(
		validateAbsence(
			state,
			draft({
				memberId: 'lucie',
				startDate: '2026-11-04',
				startPart: 'morning',
				endDate: '2026-11-04',
				endPart: 'morning',
			}),
		).length,
		0,
	)
})

test('createAbsence passe en attente si le motif demande une validation', () => {
	const created = createAbsence(state, context, draft())
	assert.equal(created.absences.length, state.absences.length + 1)
	assert.equal(created.absences[0].status, 'pending')
	assert.equal(created.absences[0].decidedById, null)
})

test('createAbsence valide directement un motif sans validation', () => {
	const created = createAbsence(
		state,
		context,
		draft({ typeId: 'maladie', proofName: 'arret.pdf' }),
	)
	assert.equal(created.absences[0].status, 'approved')
})

test('createAbsence ignore une demande invalide', () => {
	assert.equal(
		createAbsence(
			state,
			context,
			draft({ startDate: '2026-12-09', endDate: '2026-12-07' }),
		),
		state,
	)
})

test('decideAbsence ne touche que les demandes en attente', () => {
	const approved = decideAbsence(
		state,
		context,
		'abs-julien-novembre',
		'approved',
		' ok ',
	)
	const target = approved.absences.find(
		(absence) => absence.id === 'abs-julien-novembre',
	)
	assert.equal(target?.status, 'approved')
	assert.equal(target?.decisionComment, 'ok')
	assert.deepEqual(
		decideAbsence(approved, context, 'abs-julien-novembre', 'rejected', '')
			.absences,
		approved.absences,
	)
})

test('cancelAbsence annule une demande validée', () => {
	const cancelled = cancelAbsence(state, context, 'abs-karim-novembre')
	assert.equal(
		cancelled.absences.find((absence) => absence.id === 'abs-karim-novembre')
			?.status,
		'cancelled',
	)
})

test('teamConflicts liste les collègues absents en même temps', () => {
	const conflicts = teamConflicts(
		state,
		draft({
			memberId: 'baptiste',
			startDate: '2026-11-12',
			endDate: '2026-11-12',
		}),
	)
	assert.deepEqual(
		conflicts.map((conflict) => conflict.member.id),
		['karim'],
	)
})
