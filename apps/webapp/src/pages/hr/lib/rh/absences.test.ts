import { assert, test } from 'vitest'
import {
	absenceDays,
	teamConflicts,
	validateAbsence,
} from '#/pages/hr/lib/rh/absences'
import type { AbsenceDraft } from '#/pages/hr/lib/rh/types'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
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
