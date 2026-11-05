import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	clearDayOverride,
	presenceOn,
	resolveDay,
	setDayOverride,
} from '#/pages/hr/lib/rh/planning'

const state = INITIAL_RH_STATE
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}

test('jour travaillé selon la semaine type', () => {
	const day = resolveDay(state, member('thomas'), '2026-11-04')
	assert.deepEqual(day.morning, {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
		overridden: false,
	})
	assert.deepEqual(day.afternoon, day.morning)
})

test('repos le week-end et férié un jour férié', () => {
	assert.equal(
		resolveDay(state, member('thomas'), '2026-11-07').morning.kind,
		'off',
	)
	assert.equal(
		resolveDay(state, member('thomas'), '2026-11-11').morning.kind,
		'holiday',
	)
})

test('demi-journée d’absence : matin au travail, après-midi absente', () => {
	const day = resolveDay(state, member('lucie'), '2026-11-04')
	assert.equal(day.morning.kind, 'work')
	assert.equal(day.afternoon.kind, 'absence')
})

test('une demande en attente est marquée comme telle', () => {
	const day = resolveDay(state, member('julien'), '2026-11-05')
	assert.deepEqual(day.morning.kind === 'absence' && day.morning.pending, true)
})

test('un arrêt maladie couvre aussi le week-end', () => {
	assert.equal(
		resolveDay(state, member('sophie'), '2026-11-07').morning.kind,
		'off',
	)
	assert.equal(
		resolveDay(state, member('sophie'), '2026-11-06').morning.kind,
		'absence',
	)
})

test('avant l’entrée et après la fin de contrat, le membre est inactif', () => {
	assert.equal(
		resolveDay(state, member('manon'), '2026-10-09').morning.kind,
		'inactive',
	)
	assert.equal(
		resolveDay(state, member('manon'), '2026-11-30').morning.kind,
		'inactive',
	)
})

test('un forçage de jour remplace la semaine type', () => {
	const worked = resolveDay(state, member('karim'), '2026-11-07')
	assert.deepEqual(worked.morning, {
		kind: 'work',
		start: '08:00',
		end: '12:00',
		lunch: null,
		overridden: true,
	})
	assert.equal(
		resolveDay(state, member('julien'), '2026-11-23').morning.kind,
		'off',
	)
})

test('setDayOverride remplace le forçage existant, clearDayOverride le retire', () => {
	const context = {
		now: '',
		today: '2026-11-04',
		actorId: 'baptiste',
		newId: () => 'x',
	}
	const changed = setDayOverride(state, context, {
		memberId: 'karim',
		date: '2026-11-07',
		kind: 'off',
		start: '',
		end: '',
		note: '',
	})
	assert.equal(changed.overrides.length, state.overrides.length)
	assert.equal(
		resolveDay(changed, member('karim'), '2026-11-07').morning.kind,
		'off',
	)
	const cleared = clearDayOverride(changed, 'karim', '2026-11-07')
	assert.equal(cleared.overrides.length, state.overrides.length - 1)
})

test('presenceOn répartit présents, absents et repos le 5 novembre', () => {
	const presence = presenceOn(state, '2026-11-05')
	assert.deepEqual(presence.absent.map((entry) => entry.member.id).sort(), [
		'julien',
		'sophie',
	])
	assert.equal(
		presence.absent.find((entry) => entry.member.id === 'julien')?.pending,
		true,
	)
	assert.deepEqual(presence.working.map((other) => other.id).sort(), [
		'baptiste',
		'hugo',
		'karim',
		'lucie',
		'manon',
		'thomas',
		'yanis',
	])
	assert.equal(presence.off.length, 0)
})

test('presenceOn exclut les membres archivés et les contrats non commencés', () => {
	const ids = presenceOn(state, '2026-10-05').working.map((other) => other.id)
	assert.equal(ids.includes('marc'), false)
	assert.equal(ids.includes('manon'), false)
})

test('presenceOn signale la demi-journée d’absence de Lucie', () => {
	const entry = presenceOn(state, '2026-11-04').absent.find(
		(item) => item.member.id === 'lucie',
	)
	assert.equal(entry?.part, 'afternoon')
})
