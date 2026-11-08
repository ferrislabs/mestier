import { assert, test } from 'vitest'
import {
	clearPlanning,
	planDays,
	validatePlan,
	WORKING_WEEKDAYS,
} from '#/pages/hr/lib/rh/day-planning'
import {
	countPlanned,
	dayPlanOf,
	defaultPlanPeriod,
	emptyPlan,
	fromWeekdays,
	planSummary,
	toWeekdays,
} from '#/pages/hr/lib/rh/plan-draft'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import type { ActionContext, WorkDay } from '#/pages/hr/lib/rh/types'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const DAY: WorkDay = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
const context: ActionContext = {
	now: '2026-11-04T10:00:00.000Z',
	today: '2026-11-04',
	actorId: 'baptiste',
	newId: () => 'x',
}

test('defaultPlanPeriod commence aujourd’hui dans le mois courant', () => {
	assert.deepEqual(defaultPlanPeriod('2026-11-01', '2026-11-04'), {
		startDate: '2026-11-04',
		endDate: '2026-11-30',
	})
})

test('defaultPlanPeriod couvre tout un mois futur ou passé', () => {
	assert.deepEqual(defaultPlanPeriod('2026-12-01', '2026-11-04'), {
		startDate: '2026-12-01',
		endDate: '2026-12-31',
	})
	assert.deepEqual(defaultPlanPeriod('2026-10-01', '2026-11-04'), {
		startDate: '2026-10-01',
		endDate: '2026-10-31',
	})
})

test('emptyPlan prévoit du lundi au vendredi avec les horaires par défaut', () => {
	const plan = emptyPlan(
		'manon',
		{ startDate: '2026-11-24', endDate: '2026-11-27' },
		DAY,
	)
	assert.equal(plan.memberId, 'manon')
	assert.deepEqual(plan.weekdays, WORKING_WEEKDAYS)
	assert.equal(plan.kind, 'work')
	assert.equal(plan.start, '08:00')
	assert.equal(plan.end, '17:00')
	assert.deepEqual(plan.lunch, DEFAULT_LUNCH)
	assert.equal(plan.note, '')
	assert.deepEqual(validatePlan(state, plan), [])
})

test('dayPlanOf cible un seul jour', () => {
	const plan = dayPlanOf('manon', '2026-11-24', 'work', DAY)
	assert.equal(plan.startDate, '2026-11-24')
	assert.equal(plan.endDate, '2026-11-24')
	assert.deepEqual(plan.weekdays, [1])
	const planned = planDays(state, context, plan)
	const override = planned.overrides.find(
		(item) => item.memberId === 'manon' && item.date === '2026-11-24',
	)
	assert.equal(override?.kind, 'work')
	assert.equal(override?.start, '08:00')
})

test('dayPlanOf en repos ne garde pas d’horaires', () => {
	const planned = planDays(
		state,
		context,
		dayPlanOf('manon', '2026-11-25', 'off', DAY),
	)
	const override = planned.overrides.find(
		(item) => item.memberId === 'manon' && item.date === '2026-11-25',
	)
	assert.equal(override?.kind, 'off')
	assert.equal(override?.start, '')
})

test('toWeekdays et fromWeekdays convertissent les valeurs du groupe de boutons', () => {
	assert.deepEqual(toWeekdays(['0', '2', '6']), [0, 2, 6])
	assert.deepEqual(fromWeekdays([1, 3]), ['1', '3'])
	assert.deepEqual(toWeekdays([]), [])
})

test('countPlanned compte les jours déjà planifiés sur une période', () => {
	assert.equal(countPlanned(state, 'manon', '2026-11-02', '2026-11-06'), 5)
	assert.equal(countPlanned(state, 'manon', '2026-11-24', '2026-11-27'), 0)
	assert.equal(countPlanned(state, 'julien', '2026-11-02', '2026-11-06'), 0)
})

test('countPlanned compte aussi les jours de repos planifiés', () => {
	assert.equal(countPlanned(state, 'manon', '2026-11-13', '2026-11-13'), 1)
})

test('planSummary annonce le nombre de jours planifiés', () => {
	const period = { startDate: '2026-11-24', endDate: '2026-11-27' }
	assert.equal(
		planSummary(state, emptyPlan('manon', period, DAY)),
		'4 jours seront planifiés.',
	)
	assert.equal(
		planSummary(state, {
			...emptyPlan('manon', period, DAY),
			startDate: '2026-11-24',
			endDate: '2026-11-24',
		}),
		'1 jour sera planifié.',
	)
})

test('planSummary signale les jours déjà planifiés remplacés', () => {
	const plan = emptyPlan(
		'manon',
		{ startDate: '2026-11-02', endDate: '2026-11-06' },
		DAY,
	)
	assert.equal(
		planSummary(state, plan),
		'5 jours seront planifiés, dont 5 déjà planifiés (ils seront remplacés).',
	)
	assert.equal(
		planSummary(state, {
			...plan,
			startDate: '2026-11-05',
			endDate: '2026-11-07',
			weekdays: [3, 4, 5],
		}),
		'3 jours seront planifiés, dont 2 déjà planifiés (ils seront remplacés).',
	)
})

test('planSummary reste sobre quand rien n’est à planifier', () => {
	const plan = {
		...emptyPlan(
			'manon',
			{ startDate: '2026-11-24', endDate: '2026-11-27' },
			DAY,
		),
		weekdays: [],
	}
	assert.equal(planSummary(state, plan), 'Aucun jour à planifier.')
})

test('effacer la planification retire les jours de la période', () => {
	const cleared = clearPlanning(state, 'manon', '2026-11-02', '2026-11-06')
	assert.equal(countPlanned(cleared, 'manon', '2026-11-02', '2026-11-06'), 0)
	assert.equal(countPlanned(cleared, 'manon', '2026-11-09', '2026-11-20'), 9)
})
