import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	clearPlanning,
	type DayPlan,
	planDates,
	planDays,
	validatePlan,
	WORKING_WEEKDAYS,
} from '#/pages/hr/lib/rh/day-planning'
import { presenceOn, resolveDay } from '#/pages/hr/lib/rh/planning'
import type { ActionContext } from '#/pages/hr/lib/rh/types'

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
	newId: () => 'id',
}

function plan(patch: Partial<DayPlan> = {}): DayPlan {
	return {
		memberId: 'manon',
		startDate: '2026-11-23',
		endDate: '2026-11-27',
		weekdays: WORKING_WEEKDAYS,
		kind: 'work',
		start: '08:00',
		end: '16:00',
		lunch: { start: '12:00', end: '13:00' },
		note: '',
		...patch,
	}
}

test('un contrat variable laisse les jours non planifiés', () => {
	const manon = member('manon')
	assert.equal(resolveDay(state, manon, '2026-11-25').morning.kind, 'unplanned')
	assert.equal(resolveDay(state, manon, '2026-11-25').variable, true)
	assert.equal(resolveDay(state, manon, '2026-11-05').morning.kind, 'work')
	assert.equal(resolveDay(state, manon, '2026-11-13').morning.kind, 'off')
	assert.equal(resolveDay(state, manon, '2026-11-11').morning.kind, 'holiday')
	assert.equal(resolveDay(state, manon, '2026-11-14').morning.kind, 'off')
	assert.equal(resolveDay(state, manon, '2026-11-15').morning.kind, 'off')
})

test('un jour planifié sur un contrat variable n’est pas un jour modifié', () => {
	const day = resolveDay(state, member('manon'), '2026-11-05')
	assert.equal(day.morning.kind === 'work' && day.morning.overridden, false)
	const off = resolveDay(state, member('manon'), '2026-11-13')
	assert.equal(off.morning.kind === 'off' && off.morning.overridden, false)
})

test('une séquence sans travail donne un jour de repos portant son libellé', () => {
	const yanis = member('yanis')
	const cfa = resolveDay(state, yanis, '2026-11-24')
	assert.equal(cfa.morning.kind, 'off')
	assert.equal(cfa.morning.kind === 'off' && cfa.morning.label, 'CFA')
	assert.equal(resolveDay(state, yanis, '2026-11-05').morning.kind, 'work')
	const weekend = resolveDay(state, yanis, '2026-11-07')
	assert.equal(
		weekend.morning.kind === 'off' && weekend.morning.label,
		undefined,
	)
})

test('la rotation A et B de Hugo change le vendredi', () => {
	const hugo = member('hugo')
	assert.equal(resolveDay(state, hugo, '2026-11-06').morning.kind, 'work')
	assert.equal(resolveDay(state, hugo, '2026-11-13').morning.kind, 'off')
	assert.equal(resolveDay(state, hugo, '2026-11-12').morning.kind, 'work')
})

test('presenceOn range les contrats variables non planifiés à part', () => {
	const presence = presenceOn(state, '2026-11-25')
	assert.equal(
		presence.unplanned.some((m) => m.id === 'manon'),
		true,
	)
	assert.equal(
		presence.working.some((m) => m.id === 'manon'),
		false,
	)
	assert.equal(
		presence.off.some((m) => m.id === 'manon'),
		false,
	)
	const yanis = presenceOn(state, '2026-11-24')
	assert.equal(
		yanis.off.some((m) => m.id === 'yanis'),
		true,
	)
})

test('planDates filtre les jours de la semaine', () => {
	assert.deepEqual(planDates(plan({ weekdays: [0, 2] })), [
		'2026-11-23',
		'2026-11-25',
	])
})

test('validatePlan contrôle période, jours et horaires', () => {
	assert.deepEqual(validatePlan(state, plan()), [])
	assert.equal(validatePlan(state, plan({ memberId: 'inconnu' })).length, 1)
	assert.equal(validatePlan(state, plan({ endDate: '2026-11-01' })).length, 1)
	assert.equal(validatePlan(state, plan({ weekdays: [] })).length, 1)
	assert.equal(
		validatePlan(
			state,
			plan({ startDate: '2026-11-28', endDate: '2026-11-29' }),
		).length,
		1,
	)
	assert.equal(validatePlan(state, plan({ end: '07:00' })).length, 1)
	assert.deepEqual(validatePlan(state, plan({ kind: 'off', end: '07:00' })), [])
})

test('planDays planifie les jours choisis et remplace l’existant', () => {
	const next = planDays(state, context, plan())
	const manon = next.members.find((m) => m.id === 'manon')
	if (!manon) throw new Error('manon')
	assert.equal(resolveDay(next, manon, '2026-11-25').morning.kind, 'work')
	assert.equal(resolveDay(next, manon, '2026-11-27').morning.kind, 'work')
	const reset = planDays(
		next,
		context,
		plan({ kind: 'off', startDate: '2026-11-25', endDate: '2026-11-25' }),
	)
	assert.equal(resolveDay(reset, manon, '2026-11-25').morning.kind, 'off')
	assert.equal(resolveDay(reset, manon, '2026-11-26').morning.kind, 'work')
	assert.equal(planDays(state, context, plan({ end: '07:00' })), state)
})

test('clearPlanning retire les jours planifiés de la période', () => {
	const next = clearPlanning(state, 'manon', '2026-11-02', '2026-11-06')
	const manon = next.members.find((m) => m.id === 'manon')
	if (!manon) throw new Error('manon')
	assert.equal(resolveDay(next, manon, '2026-11-04').morning.kind, 'unplanned')
	assert.equal(resolveDay(next, manon, '2026-11-09').morning.kind, 'work')
})
