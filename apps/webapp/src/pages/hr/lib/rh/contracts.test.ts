import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	contractOn,
	contractTrail,
	currentContract,
	hireDateOf,
	leaveDateOf,
	phaseOf,
	scheduleOn,
	suggestNextContract,
	validateContracts,
} from '#/pages/hr/lib/rh/contracts'
import {
	addContract,
	contractErrors,
	removeContract,
	setContractSchedule,
	upcomingDeadlines,
	updateContract,
} from '#/pages/hr/lib/rh/members'
import { isEmployedOn, resolveDay } from '#/pages/hr/lib/rh/planning'
import type { ActionContext, ContractDraft } from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}
const context: ActionContext = {
	now: '2026-11-04T10:00:00.000Z',
	today: '2026-11-04',
	actorId: 'baptiste',
	newId: () => 'c-new',
}

function draft(patch: Partial<ContractDraft> = {}): ContractDraft {
	return {
		type: 'cdi',
		startDate: '2027-01-01',
		endDate: null,
		trialEndDate: null,
		schedule: member('karim').contracts[0].schedule,
		...patch,
	}
}

test('Julien enchaîne apprentissage, CDD puis CDI', () => {
	const julien = member('julien')
	assert.deepEqual(contractTrail(julien), ['apprentissage', 'cdd', 'cdi'])
	assert.equal(hireDateOf(julien), '2019-09-02')
	assert.equal(leaveDateOf(julien), null)
	assert.equal(currentContract(julien, '2026-11-04').type, 'cdi')
	assert.equal(currentContract(julien, '2021-12-01').type, 'cdd')
	assert.equal(contractOn(julien, '2019-08-31'), null)
})

test('un avenant de même type ne répète pas le type dans le parcours', () => {
	assert.deepEqual(contractTrail(member('lucie')), ['cdi'])
	assert.equal(member('lucie').contracts.length, 2)
})

test('currentContract retombe sur le dernier contrat démarré ou le premier à venir', () => {
	assert.equal(currentContract(member('marc'), '2026-11-04').id, 'c-marc-1')
	assert.equal(currentContract(member('manon'), '2026-10-01').id, 'c-manon-1')
})

test('phaseOf', () => {
	const [first, , third] = member('julien').contracts
	assert.equal(phaseOf(first, '2026-11-04'), 'ended')
	assert.equal(phaseOf(third, '2026-11-04'), 'current')
	assert.equal(phaseOf(third, '2021-01-01'), 'upcoming')
})

test('la semaine type suit le contrat en vigueur à la date', () => {
	const lucie = member('lucie')
	assert.equal(scheduleOn(lucie, '2022-06-15')[3] !== null, true)
	assert.equal(scheduleOn(lucie, '2026-11-05')[3] !== null, true)
	assert.equal(scheduleOn(lucie, '2022-06-17')[4] !== null, true)
	assert.equal(scheduleOn(lucie, '2026-11-06')[4], null)
	assert.equal(
		scheduleOn(lucie, '2020-01-01').every((day) => day === null),
		true,
	)
})

test('le planning ne change pas rétroactivement après un avenant', () => {
	const lucie = member('lucie')
	assert.equal(resolveDay(state, lucie, '2022-06-17').morning.kind, 'work')
	assert.equal(resolveDay(state, lucie, '2026-11-06').morning.kind, 'off')
})

test('entre deux contrats ou après le dernier, le membre n’est pas employé', () => {
	const gap = {
		...member('julien'),
		contracts: member('julien').contracts.filter((c) => c.id !== 'c-julien-2'),
	}
	assert.equal(isEmployedOn(gap, '2021-12-01'), false)
	assert.equal(isEmployedOn(member('julien'), '2021-12-01'), true)
	assert.equal(isEmployedOn(member('marc'), '2026-07-15'), false)
})

test('validateContracts exige début, fin pour les contrats à durée limitée et pas de chevauchement', () => {
	assert.deepEqual(validateContracts([draft()]), [])
	assert.equal(validateContracts([]).length, 1)
	assert.equal(validateContracts([draft({ startDate: '' })]).length, 1)
	assert.equal(validateContracts([draft({ type: 'cdd' })]).length, 1)
	assert.equal(
		validateContracts([draft({ type: 'cdd', endDate: '2026-12-31' })]).length,
		1,
	)
	assert.equal(
		validateContracts([draft({ trialEndDate: '2026-12-31' })]).length,
		1,
	)
	assert.equal(
		validateContracts([
			draft({ type: 'cdd', startDate: '2026-01-01', endDate: '2026-06-30' }),
			draft({ startDate: '2026-06-30' }),
		]).length,
		1,
	)
	assert.equal(
		validateContracts([
			draft({ startDate: '2026-01-01' }),
			draft({ startDate: '2027-01-01' }),
		]).length,
		1,
	)
	assert.deepEqual(
		validateContracts([
			draft({ type: 'cdd', startDate: '2026-01-01', endDate: '2026-06-30' }),
			draft({ startDate: '2026-07-01' }),
		]),
		[],
	)
})

test('addContract ajoute un contrat à la suite et le garde trié', () => {
	const julien = member('julien')
	const closed = updateContract(state, 'karim', 'c-karim-1', {
		endDate: '2026-12-31',
		type: 'cdd',
	})
	const next = addContract(
		closed,
		context,
		'karim',
		draft({ startDate: '2027-01-01' }),
	)
	const contracts = next.members.find((m) => m.id === 'karim')?.contracts ?? []
	assert.deepEqual(
		contracts.map((c) => c.id),
		['c-karim-1', 'c-new'],
	)
	assert.equal(
		next.members.find((m) => m.id === 'julien'),
		julien,
	)
})

test('addContract refuse un chevauchement', () => {
	assert.equal(
		addContract(state, context, 'karim', draft({ startDate: '2026-01-01' })),
		state,
	)
})

test('removeContract garde au moins un contrat', () => {
	assert.equal(removeContract(state, 'karim', 'c-karim-1'), state)
	const next = removeContract(state, 'julien', 'c-julien-2')
	assert.equal(next.members.find((m) => m.id === 'julien')?.contracts.length, 2)
})

test('setContractSchedule ne modifie que le contrat visé', () => {
	const schedule = member('thomas').contracts[1].schedule
	const next = setContractSchedule(state, 'julien', 'c-julien-1', schedule)
	const contracts = next.members.find((m) => m.id === 'julien')?.contracts ?? []
	assert.equal(contracts[0].schedule, schedule)
	assert.equal(contracts[2].schedule, member('julien').contracts[2].schedule)
})

test('contractErrors ignore le contrat édité', () => {
	const julien = member('julien')
	const edited = { ...julien.contracts[1], id: undefined }
	assert.deepEqual(contractErrors(julien, edited, 'c-julien-2'), [])
	assert.equal(contractErrors(julien, edited).length, 1)
})

test('suggestNextContract démarre le lendemain du dernier contrat', () => {
	const hugo = member('hugo')
	const suggested = suggestNextContract(hugo, '2026-11-04')
	assert.equal(suggested.startDate, '2026-12-19')
	assert.equal(suggested.type, 'cdi')
	assert.equal(suggested.schedule, hugo.contracts[0].schedule)
	assert.equal(
		suggestNextContract(member('julien'), '2026-11-04').startDate,
		'2026-11-04',
	)
})

test('la fin de contrat n’est une échéance que pour le dernier contrat', () => {
	const kinds = upcomingDeadlines(state, '2019-10-01', 60).filter(
		(deadline) => deadline.memberId === 'julien',
	)
	assert.deepEqual(kinds, [])
	const closed = updateContract(state, 'hugo', 'c-hugo-1', {
		endDate: '2026-12-01',
	})
	const withNext = addContract(
		closed,
		context,
		'hugo',
		draft({ startDate: '2026-12-02' }),
	)
	const hugo = upcomingDeadlines(withNext, '2026-11-04').filter(
		(d) => d.memberId === 'hugo',
	)
	assert.deepEqual(
		hugo.map((d) => d.kind),
		['work_permit'],
	)
})
