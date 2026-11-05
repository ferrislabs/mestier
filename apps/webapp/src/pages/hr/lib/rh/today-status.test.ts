import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { weekly } from '#/pages/hr/lib/rh/contract-schedule'
import { archiveMember } from '#/pages/hr/lib/rh/members'
import { todayStatus } from '#/pages/hr/lib/rh/today-status'
import type { ActionContext, Member, RhState } from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE

function memberOf(id: string): Member {
	const found = state.members.find((member) => member.id === id)
	if (!found) throw new Error(id)
	return found
}

test('un jour travaillé donne les horaires et la pause', () => {
	assert.deepEqual(todayStatus(state, memberOf('karim'), '2026-11-04'), {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
	})
})

test('la pause propre au membre est reprise', () => {
	assert.deepEqual(todayStatus(state, memberOf('lucie'), '2026-11-03'), {
		kind: 'work',
		start: '09:00',
		end: '17:30',
		lunch: { start: '12:30', end: '13:30' },
	})
})

test('un jour sans pause donne lunch null', () => {
	const noLunch: RhState = {
		...state,
		members: state.members.map((member) =>
			member.id === 'karim'
				? {
						...member,
						contracts: member.contracts.map((contract) => ({
							...contract,
							schedule:
								contract.schedule.kind === 'weekly'
									? weekly(
											contract.schedule.week.map((day) =>
												day ? { ...day, lunch: null } : day,
											),
										)
									: contract.schedule,
						})),
					}
				: member,
		),
	}
	const karim = noLunch.members.find((member) => member.id === 'karim')
	assert.ok(karim)
	assert.deepEqual(todayStatus(noLunch, karim, '2026-11-04'), {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: null,
	})
})

test('un jour non travaillé donne repos', () => {
	assert.deepEqual(todayStatus(state, memberOf('karim'), '2026-11-08'), {
		kind: 'off',
	})
	assert.deepEqual(todayStatus(state, memberOf('lucie'), '2026-11-06'), {
		kind: 'off',
	})
})

test('un jour férié est signalé', () => {
	assert.deepEqual(todayStatus(state, memberOf('karim'), '2026-11-11'), {
		kind: 'holiday',
		name: 'Armistice 1918',
	})
})

test('une absence validée sur la journée donne le motif', () => {
	assert.deepEqual(todayStatus(state, memberOf('sophie'), '2026-11-04'), {
		kind: 'absence',
		typeId: 'maladie',
		part: 'full',
		pending: false,
	})
})

test('une absence de l’après-midi laisse la matinée travaillée', () => {
	assert.deepEqual(todayStatus(state, memberOf('lucie'), '2026-11-04'), {
		kind: 'absence',
		typeId: 'cp',
		part: 'afternoon',
		pending: false,
	})
})

test('une demande en attente est signalée', () => {
	assert.deepEqual(todayStatus(state, memberOf('julien'), '2026-11-05'), {
		kind: 'absence',
		typeId: 'cp',
		part: 'full',
		pending: true,
	})
})

test('une absence sur une demi-journée garde la part concernée', () => {
	const partial: RhState = {
		...state,
		absences: [
			{
				...state.absences[0],
				id: 'partial',
				memberId: 'karim',
				typeId: 'cp',
				startDate: '2026-11-04',
				startPart: 'afternoon',
				endDate: '2026-11-04',
				endPart: 'afternoon',
				status: 'approved',
			},
		],
	}
	assert.deepEqual(todayStatus(partial, memberOf('karim'), '2026-11-04'), {
		kind: 'absence',
		typeId: 'cp',
		part: 'afternoon',
		pending: false,
	})
})

test('un membre pas encore arrivé ou archivé est inactif', () => {
	assert.deepEqual(todayStatus(state, memberOf('manon'), '2026-10-01'), {
		kind: 'inactive',
	})
	assert.deepEqual(todayStatus(state, memberOf('marc'), '2026-11-04'), {
		kind: 'inactive',
	})
})

test('un membre archivé aujourd’hui est inactif', () => {
	const context: ActionContext = {
		now: '2026-11-04T10:00:00.000Z',
		today: '2026-11-04',
		actorId: 'baptiste',
		newId: () => 'x',
	}
	const archived = archiveMember(state, context, 'thomas')
	const thomas = archived.members.find((member) => member.id === 'thomas')
	assert.ok(thomas)
	assert.deepEqual(todayStatus(archived, thomas, '2026-11-04'), {
		kind: 'inactive',
	})
})

test('la semaine appliquée est celle du contrat en vigueur ce jour-là', () => {
	assert.deepEqual(todayStatus(state, memberOf('julien'), '2021-06-14'), {
		kind: 'work',
		start: '08:00',
		end: '16:00',
		lunch: { start: '12:00', end: '13:00' },
	})
	assert.deepEqual(todayStatus(state, memberOf('julien'), '2026-11-04'), {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
	})
})

test('un changement de contrat modifie les jours travaillés', () => {
	assert.deepEqual(todayStatus(state, memberOf('lucie'), '2023-06-02'), {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
	})
	assert.deepEqual(todayStatus(state, memberOf('lucie'), '2023-09-08'), {
		kind: 'off',
	})
})

test('un jour de CFA donne le libellé de la séquence', () => {
	assert.deepEqual(todayStatus(state, memberOf('yanis'), '2026-11-24'), {
		kind: 'off',
		label: 'CFA',
	})
	assert.equal(todayStatus(state, memberOf('yanis'), '2026-11-04').kind, 'work')
})

test('un jour de contrat variable non planifié est à planifier', () => {
	assert.deepEqual(todayStatus(state, memberOf('manon'), '2026-11-24'), {
		kind: 'unplanned',
	})
	assert.equal(todayStatus(state, memberOf('manon'), '2026-11-04').kind, 'work')
	assert.deepEqual(todayStatus(state, memberOf('manon'), '2026-11-13'), {
		kind: 'off',
	})
})

test('la semaine B de Hugo laisse le vendredi en repos', () => {
	assert.deepEqual(todayStatus(state, memberOf('hugo'), '2026-11-13'), {
		kind: 'off',
	})
	assert.equal(todayStatus(state, memberOf('hugo'), '2026-11-06').kind, 'work')
})
