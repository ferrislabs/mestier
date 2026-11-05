import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	averageWeeklyHours,
	describeCycle,
	NO_SCHEDULE,
	validateSchedule,
	weekOn,
} from '#/pages/hr/lib/rh/contract-schedule'
import { shiftMonths } from '#/pages/hr/lib/rh/dates'
import { blockOn, previewBlocks } from '#/pages/hr/lib/rh/rotation'
import type {
	ContractSchedule,
	ScheduleBlock,
	WeekSchedule,
} from '#/pages/hr/lib/rh/types'

const FULL: WeekSchedule = [
	{ start: '08:00', end: '17:00', lunch: { start: '12:00', end: '13:00' } },
	{ start: '08:00', end: '17:00', lunch: { start: '12:00', end: '13:00' } },
	{ start: '08:00', end: '17:00', lunch: { start: '12:00', end: '13:00' } },
	{ start: '08:00', end: '17:00', lunch: { start: '12:00', end: '13:00' } },
	{ start: '08:00', end: '17:00', lunch: { start: '12:00', end: '13:00' } },
	null,
	null,
]

function blocks(): ScheduleBlock[] {
	return [
		{ length: 2, unit: 'weeks', label: 'Entreprise', week: FULL },
		{ length: 1, unit: 'weeks', label: 'CFA', week: NO_SCHEDULE },
	]
}

function rotation(
	list: ScheduleBlock[] = blocks(),
	anchor = '2026-01-05',
): Extract<ContractSchedule, { kind: 'rotation' }> {
	return { kind: 'rotation', anchor, blocks: list }
}

test('une rotation de semaines répète ses séquences à partir du point de départ', () => {
	const schedule = rotation()
	assert.equal(blockOn(schedule, '2026-01-05')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-01-18')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-01-19')?.block.label, 'CFA')
	assert.equal(blockOn(schedule, '2026-01-25')?.block.label, 'CFA')
	assert.equal(blockOn(schedule, '2026-01-26')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-02-09')?.block.label, 'CFA')
})

test('la séquence couvre ses bornes exactes', () => {
	const span = blockOn(rotation(), '2026-01-21')
	assert.equal(span?.start, '2026-01-19')
	assert.equal(span?.end, '2026-01-25')
	assert.equal(span?.index, 1)
})

test('avant le point de départ, le cycle se prolonge vers le passé', () => {
	const schedule = rotation()
	assert.equal(blockOn(schedule, '2026-01-04')?.block.label, 'CFA')
	assert.equal(blockOn(schedule, '2025-12-22')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2025-12-21')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2025-12-14')?.block.label, 'CFA')
})

test('une rotation en mois suit les mois calendaires', () => {
	const schedule = rotation(
		[
			{ length: 3, unit: 'months', label: 'Entreprise', week: FULL },
			{ length: 1, unit: 'months', label: 'École', week: NO_SCHEDULE },
		],
		'2026-01-01',
	)
	assert.equal(blockOn(schedule, '2026-03-31')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-04-01')?.block.label, 'École')
	assert.equal(blockOn(schedule, '2026-04-30')?.block.label, 'École')
	assert.equal(blockOn(schedule, '2026-05-01')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-09-15')?.block.label, 'Entreprise')
	assert.equal(blockOn(schedule, '2026-08-15')?.block.label, 'École')
	assert.equal(blockOn(schedule, '2025-12-15')?.block.label, 'École')
	assert.equal(blockOn(schedule, '2025-09-15')?.block.label, 'Entreprise')
})

test('une rotation mêlant semaines et mois reste cohérente', () => {
	const schedule = rotation(
		[
			{ length: 2, unit: 'weeks', label: 'A', week: FULL },
			{ length: 1, unit: 'months', label: 'B', week: NO_SCHEDULE },
		],
		'2026-01-05',
	)
	assert.equal(blockOn(schedule, '2026-01-19')?.block.label, 'B')
	assert.equal(blockOn(schedule, '2026-02-18')?.block.label, 'B')
	assert.equal(blockOn(schedule, '2026-02-19')?.block.label, 'A')
})

test('shiftMonths conserve le jour ou se cale en fin de mois', () => {
	assert.equal(shiftMonths('2026-01-15', 1), '2026-02-15')
	assert.equal(shiftMonths('2026-01-31', 1), '2026-02-28')
	assert.equal(shiftMonths('2026-03-31', -1), '2026-02-28')
})

test('weekOn renvoie la semaine du bloc et son libellé', () => {
	const schedule = rotation()
	assert.equal(weekOn(schedule, '2026-01-07').week, FULL)
	assert.equal(weekOn(schedule, '2026-01-07').label, 'Entreprise')
	assert.equal(weekOn(schedule, '2026-01-21').label, 'CFA')
	assert.equal(weekOn({ kind: 'weekly', week: FULL }, '2026-01-21').label, null)
	const variable = weekOn(
		{ kind: 'variable', defaultDay: FULL[0] as NonNullable<WeekSchedule[0]> },
		'2026-01-21',
	)
	assert.equal(variable.week[4] !== null, true)
	assert.equal(variable.week[5], null)
})

test('averageWeeklyHours pondère par la durée de chaque séquence', () => {
	assert.equal(averageWeeklyHours({ kind: 'weekly', week: FULL }), 40)
	assert.equal(averageWeeklyHours(rotation()), 26.7)
	assert.equal(
		averageWeeklyHours({
			kind: 'variable',
			defaultDay: FULL[0] as NonNullable<WeekSchedule[0]>,
		}),
		null,
	)
})

test('previewBlocks enchaîne les séquences suivantes', () => {
	const spans = previewBlocks(rotation(), '2026-01-21', 3)
	assert.deepEqual(
		spans.map((span) => [span.block.label, span.start, span.end]),
		[
			['CFA', '2026-01-19', '2026-01-25'],
			['Entreprise', '2026-01-26', '2026-02-08'],
			['CFA', '2026-02-09', '2026-02-15'],
		],
	)
})

test('describeCycle résume le cycle', () => {
	assert.equal(
		describeCycle(blocks()),
		'Entreprise (2 semaines) puis CFA (1 semaine)',
	)
})

test('validateSchedule contrôle les rotations', () => {
	assert.deepEqual(validateSchedule(rotation()), [])
	assert.equal(validateSchedule(rotation([])).length, 1)
	assert.equal(
		validateSchedule(
			rotation([{ length: 0, unit: 'weeks', label: '', week: FULL }]),
		).length,
		1,
	)
	assert.equal(
		validateSchedule(
			rotation([{ length: 13, unit: 'months', label: '', week: FULL }]),
		).length,
		1,
	)
	assert.equal(
		validateSchedule(
			rotation(
				[{ length: 1, unit: 'months', label: '', week: FULL }],
				'2026-01-30',
			),
		).length,
		1,
	)
	assert.equal(validateSchedule(rotation(blocks(), '')).length, 1)
	const bad: WeekSchedule = [
		{ start: '17:00', end: '08:00' },
		null,
		null,
		null,
		null,
		null,
		null,
	]
	assert.equal(validateSchedule({ kind: 'weekly', week: bad }).length, 1)
})

test('les données de démo : Yanis alterne CFA et entreprise, Hugo semaine A et B', () => {
	const state = INITIAL_RH_STATE
	const yanis = state.members.find((m) => m.id === 'yanis')
	const hugo = state.members.find((m) => m.id === 'hugo')
	if (!yanis || !hugo) throw new Error('membres absents')
	const [ySchedule, hSchedule] = [
		yanis.contracts[0].schedule,
		hugo.contracts[0].schedule,
	]
	if (ySchedule.kind !== 'rotation' || hSchedule.kind !== 'rotation')
		throw new Error('rotation attendue')
	assert.equal(blockOn(ySchedule, '2026-11-04')?.block.label, 'Entreprise')
	assert.equal(blockOn(ySchedule, '2026-11-23')?.block.label, 'CFA')
	assert.equal(blockOn(hSchedule, '2026-11-06')?.block.label, 'Semaine A')
	assert.equal(blockOn(hSchedule, '2026-11-13')?.block.label, 'Semaine B')
})
