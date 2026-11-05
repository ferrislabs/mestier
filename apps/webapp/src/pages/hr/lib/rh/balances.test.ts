import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	acquiredAt,
	addAdjustment,
	computeBalance,
	movementsFor,
	periodOf,
	validateAdjustment,
} from '#/pages/hr/lib/rh/balances'

const state = INITIAL_RH_STATE
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}
const account = (id: string) => {
	const found = state.accounts.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}
const AS_OF = '2026-11-04'

test('la période des congés payés démarre au 1er juin', () => {
	assert.deepEqual(periodOf(account('cp'), AS_OF), {
		start: '2026-06-01',
		end: '2027-05-31',
	})
	assert.deepEqual(periodOf(account('cp'), '2026-03-10'), {
		start: '2025-06-01',
		end: '2026-05-31',
	})
	assert.deepEqual(periodOf(account('rtt'), AS_OF), {
		start: '2026-01-01',
		end: '2026-12-31',
	})
})

test('acquisition mensuelle arrondie au demi-jour', () => {
	const period = periodOf(account('cp'), AS_OF)
	assert.equal(acquiredAt(account('cp'), member('karim'), period, AS_OF), 12.5)
	assert.equal(
		acquiredAt(account('cp'), member('karim'), period, period.end),
		25,
	)
})

test('acquisition au prorata de la date d’entrée', () => {
	const period = periodOf(account('cp'), AS_OF)
	assert.equal(acquiredAt(account('cp'), member('sophie'), period, AS_OF), 10.5)
	const rtt = periodOf(account('rtt'), AS_OF)
	assert.equal(acquiredAt(account('rtt'), member('sophie'), rtt, AS_OF), 5)
	assert.equal(acquiredAt(account('rtt'), member('thomas'), rtt, AS_OF), 10)
})

test('solde de Karim : acquis, report, pris, posé et disponible', () => {
	const line = computeBalance(state, member('karim'), account('cp'), AS_OF)
	assert.equal(line.acquired, 12.5)
	assert.equal(line.carriedOver, 5)
	assert.equal(line.taken, 5)
	assert.equal(line.planned, 4)
	assert.equal(line.pending, 0)
	assert.equal(line.balance, 12.5)
	assert.equal(line.available, 8.5)
	assert.equal(line.forecast, 25 + 5 - 5 - 4)
})

test('les demandes en attente réduisent le disponible mais pas le solde', () => {
	const line = computeBalance(state, member('thomas'), account('rtt'), AS_OF)
	assert.equal(line.taken, 0)
	assert.equal(line.adjusted, -0.5)
	assert.equal(line.planned, 1)
	assert.equal(line.pending, 1)
	assert.equal(line.balance, 9.5)
	assert.equal(line.available, 7.5)
})

test('une absence refusée ne consomme rien', () => {
	const line = computeBalance(state, member('thomas'), account('cp'), AS_OF)
	assert.equal(line.pending, 0)
	assert.equal(line.planned, 0)
})

test('movementsFor liste acquisition, report, régularisation et absences, les plus récents d’abord', () => {
	const movements = movementsFor(state, member('lucie'), account('cp'), AS_OF)
	assert.deepEqual(movements.map((movement) => movement.kind).sort(), [
		'absence',
		'absence',
		'acquisition',
		'adjustment',
		'carry_over',
	])
	const dates = movements.map((movement) => movement.date)
	assert.deepEqual(dates, [...dates].sort().reverse())
})

test('validateAdjustment borne le report et exige un motif', () => {
	const base = {
		memberId: 'karim',
		accountId: 'cp',
		date: AS_OF,
		days: 3,
		kind: 'adjustment' as const,
		reason: 'Geste commercial',
	}
	assert.deepEqual(validateAdjustment(state, base), [])
	assert.equal(validateAdjustment(state, { ...base, reason: ' ' }).length, 1)
	assert.equal(validateAdjustment(state, { ...base, days: 0 }).length, 1)
	assert.equal(
		validateAdjustment(state, { ...base, kind: 'carry_over', days: 6 }).length,
		1,
	)
	assert.equal(
		validateAdjustment(state, {
			...base,
			accountId: 'rtt',
			kind: 'carry_over',
			days: 1,
		}).length,
		1,
	)
})

test('addAdjustment ajoute une régularisation signée', () => {
	const context = {
		now: '',
		today: AS_OF,
		actorId: 'baptiste',
		newId: () => 'adj-new',
	}
	const next = addAdjustment(state, context, {
		memberId: 'karim',
		accountId: 'cp',
		date: AS_OF,
		days: -1,
		kind: 'adjustment',
		reason: 'Jour pris en trop en août',
	})
	assert.equal(next.adjustments.length, state.adjustments.length + 1)
	assert.equal(
		computeBalance(next, member('karim'), account('cp'), AS_OF).balance,
		11.5,
	)
})
