import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	NO_SCHEDULE,
	nominalWeek,
	validateSchedule,
	weekly,
} from '#/pages/hr/lib/rh/contract-schedule'
import { defaultSchedule } from '#/pages/hr/lib/rh/members'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import {
	appendBlock,
	blockLengthError,
	columnSummary,
	convertSchedule,
	cycleLabel,
	duplicateBlock,
	firstWorkedDay,
	followStart,
	groupScheduleErrors,
	moveBlock,
	patchBlock,
	previewLines,
	removeBlock,
	rhythmLines,
	scheduleVolume,
	variableMonthSummary,
	weekSummary,
} from '#/pages/hr/lib/rh/schedule-mode'
import type {
	ContractSchedule,
	ScheduleBlock,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE
const TODAY = '2026-11-04'
const DAY: WorkDay = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
const SHORT: WorkDay = { start: '09:00', end: '12:00', lunch: null }

function block(
	label: string,
	week: WeekSchedule,
	length = 1,
	unit: ScheduleBlock['unit'] = 'weeks',
): ScheduleBlock {
	return { length, unit, label, week }
}

function rotation(
	blocks: ScheduleBlock[],
	anchor = '2026-11-02',
): ContractSchedule {
	return { kind: 'rotation', anchor, blocks }
}

const FULL = defaultSchedule()
const THREE_DAYS: WeekSchedule = [DAY, DAY, DAY, null, null, null, null]

test('firstWorkedDay trouve le premier jour travaillé de chaque mode', () => {
	assert.deepEqual(firstWorkedDay(weekly(FULL)), DAY)
	assert.equal(firstWorkedDay(weekly(NO_SCHEDULE)), null)
	assert.deepEqual(
		firstWorkedDay(
			rotation([block('CFA', NO_SCHEDULE), block('Entreprise', THREE_DAYS)]),
		),
		DAY,
	)
	assert.deepEqual(
		firstWorkedDay({ kind: 'variable', defaultDay: SHORT }),
		SHORT,
	)
})

test('convertSchedule régulier vers rotation garde la semaine dans une séquence d’une semaine', () => {
	const result = convertSchedule(weekly(THREE_DAYS), 'rotation', '2026-09-01')
	assert.deepEqual(result, {
		kind: 'rotation',
		anchor: '2026-09-01',
		blocks: [{ length: 1, unit: 'weeks', label: '', week: THREE_DAYS }],
	})
})

test('convertSchedule rotation vers régulier reprend la première séquence', () => {
	const result = convertSchedule(
		rotation([block('A', THREE_DAYS), block('B', FULL)]),
		'weekly',
		'2026-09-01',
	)
	assert.deepEqual(result, weekly(THREE_DAYS))
})

test('convertSchedule vers variable prend le premier jour travaillé', () => {
	assert.deepEqual(
		convertSchedule(
			weekly([null, SHORT, DAY, null, null, null, null]),
			'variable',
			'2026-09-01',
		),
		{ kind: 'variable', defaultDay: SHORT },
	)
	assert.deepEqual(
		convertSchedule(
			rotation([block('CFA', NO_SCHEDULE), block('Entreprise', THREE_DAYS)]),
			'variable',
			'2026-09-01',
		),
		{ kind: 'variable', defaultDay: DAY },
	)
})

test('convertSchedule vers variable retombe sur 8h–17h avec pause sans jour travaillé', () => {
	assert.deepEqual(
		convertSchedule(weekly(NO_SCHEDULE), 'variable', '2026-09-01'),
		{
			kind: 'variable',
			defaultDay: { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH },
		},
	)
})

test('convertSchedule depuis variable construit une semaine nominale', () => {
	const variable: ContractSchedule = { kind: 'variable', defaultDay: SHORT }
	assert.deepEqual(
		convertSchedule(variable, 'weekly', '2026-09-01'),
		weekly(nominalWeek(SHORT)),
	)
	assert.deepEqual(convertSchedule(variable, 'rotation', '2026-09-01'), {
		kind: 'rotation',
		anchor: '2026-09-01',
		blocks: [{ length: 1, unit: 'weeks', label: '', week: nominalWeek(SHORT) }],
	})
})

test('convertSchedule vers le même mode ne change rien', () => {
	const current = weekly(FULL)
	assert.equal(convertSchedule(current, 'weekly', '2026-09-01'), current)
})

test('les conversions produisent des rythmes valides', () => {
	for (const from of ['weekly', 'rotation', 'variable'] as const) {
		const start = convertSchedule(weekly(FULL), from, '2026-09-01')
		for (const to of ['weekly', 'rotation', 'variable'] as const) {
			assert.deepEqual(
				validateSchedule(convertSchedule(start, to, '2026-09-01')),
				[],
			)
		}
	}
})

test('followStart déplace le début du cycle seulement s’il suivait le contrat', () => {
	const synced = rotation([block('A', FULL)], '2026-09-01')
	const moved = followStart(synced, '2026-09-01', '2026-10-05')
	assert.equal(moved.kind === 'rotation' && moved.anchor, '2026-10-05')
	const custom = rotation([block('A', FULL)], '2026-09-14')
	assert.equal(followStart(custom, '2026-09-01', '2026-10-05'), custom)
	const regular = weekly(FULL)
	assert.equal(followStart(regular, '2026-09-01', '2026-10-05'), regular)
})

test('appendBlock copie la semaine de la dernière séquence', () => {
	const blocks = [block('A', FULL), block('B', THREE_DAYS, 2)]
	const next = appendBlock(blocks)
	assert.equal(next.length, 3)
	assert.deepEqual(next[2], {
		length: 1,
		unit: 'weeks',
		label: '',
		week: THREE_DAYS,
	})
	assert.equal(blocks.length, 2)
})

test('appendBlock sur un cycle vide utilise la semaine par défaut', () => {
	assert.deepEqual(appendBlock([])[0].week, defaultSchedule())
})

test('appendBlock s’arrête au nombre maximal de séquences', () => {
	const full = Array.from({ length: 12 }, (_, index) =>
		block(`S${index}`, FULL),
	)
	assert.equal(appendBlock(full).length, 12)
})

test('duplicateBlock insère une copie juste après la séquence', () => {
	const blocks = [block('A', FULL), block('B', THREE_DAYS, 2)]
	const next = duplicateBlock(blocks, 0)
	assert.deepEqual(
		next.map((item) => item.label),
		['A', 'A', 'B'],
	)
	assert.notEqual(next[0], next[1])
})

test('moveBlock échange deux séquences voisines et borne les extrémités', () => {
	const blocks = [block('A', FULL), block('B', FULL), block('C', FULL)]
	assert.deepEqual(
		moveBlock(blocks, 1, -1).map((item) => item.label),
		['B', 'A', 'C'],
	)
	assert.deepEqual(
		moveBlock(blocks, 1, 1).map((item) => item.label),
		['A', 'C', 'B'],
	)
	assert.equal(moveBlock(blocks, 0, -1), blocks)
	assert.equal(moveBlock(blocks, 2, 1), blocks)
})

test('removeBlock retire une séquence', () => {
	const blocks = [block('A', FULL), block('B', FULL)]
	assert.deepEqual(
		removeBlock(blocks, 0).map((item) => item.label),
		['B'],
	)
})

test('patchBlock modifie une seule séquence', () => {
	const blocks = [block('A', FULL), block('B', FULL)]
	const next = patchBlock(blocks, 1, { length: 3, unit: 'months' })
	assert.equal(next[1].length, 3)
	assert.equal(next[1].unit, 'months')
	assert.equal(next[0], blocks[0])
})

test('blockLengthError reprend les limites du socle', () => {
	assert.equal(blockLengthError(block('A', FULL, 2)), null)
	assert.equal(
		blockLengthError(block('A', FULL, 0)),
		'Une séquence dure entre 1 et 52 semaines.',
	)
	assert.equal(
		blockLengthError(block('A', FULL, 13, 'months')),
		'Une séquence dure entre 1 et 12 mois.',
	)
	assert.equal(
		blockLengthError(block('A', FULL, 1.5)),
		'Une séquence dure entre 1 et 52 semaines.',
	)
})

test('groupScheduleErrors range les messages de rotation', () => {
	const grouped = groupScheduleErrors([
		'Renseigne la date de début du cycle.',
		'Ajoute au moins une séquence au cycle.',
		'Pour des séquences en mois, démarre le cycle avant le 29 du mois.',
		'Une séquence dure entre 1 et 52 semaines.',
		'La fin de journée doit suivre le début.',
	])
	assert.equal(grouped.anchor, 'Renseigne la date de début du cycle.')
	assert.equal(grouped.blocks, 'Ajoute au moins une séquence au cycle.')
	assert.deepEqual(grouped.other, [
		'Une séquence dure entre 1 et 52 semaines.',
		'La fin de journée doit suivre le début.',
	])
	assert.equal(
		groupScheduleErrors([
			'Pour des séquences en mois, démarre le cycle avant le 29 du mois.',
		]).anchor,
		'Pour des séquences en mois, démarre le cycle avant le 29 du mois.',
	)
})

test('weekSummary donne les heures nettes et les jours', () => {
	assert.equal(weekSummary(FULL), '40 h · Lun–Ven')
	assert.equal(weekSummary(THREE_DAYS), '24 h · Lun–Mer')
	assert.equal(weekSummary(NO_SCHEDULE), 'Pas de travail')
})

test('scheduleVolume décrit chaque mode', () => {
	assert.equal(scheduleVolume(weekly(FULL)), '40 h par semaine')
	assert.equal(
		scheduleVolume({ kind: 'variable', defaultDay: DAY }),
		'Planning variable',
	)
	const cycle = rotation([
		block('Entreprise', FULL, 3),
		block('CFA', NO_SCHEDULE),
	])
	assert.equal(scheduleVolume(cycle), '≈ 30 h par semaine en moyenne')
	assert.equal(
		scheduleVolume(cycle, true),
		'≈ 30 h par semaine en moyenne · rotation de 4 semaines',
	)
})

test('cycleLabel arrondit la durée du cycle', () => {
	assert.equal(
		cycleLabel([block('A', FULL, 1), block('B', FULL, 1)]),
		'rotation de 2 semaines',
	)
	assert.equal(cycleLabel([block('A', FULL, 1)]), 'rotation de 1 semaine')
	assert.equal(
		cycleLabel([
			block('Entreprise', FULL, 3, 'months'),
			block('École', NO_SCHEDULE, 1, 'months'),
		]),
		'rotation de 17 semaines',
	)
})

test('columnSummary adapte la colonne de la liste à chaque mode', () => {
	assert.deepEqual(columnSummary(weekly(FULL), TODAY), {
		hours: '40 h',
		tag: null,
		week: FULL,
	})
	assert.deepEqual(
		columnSummary({ kind: 'variable', defaultDay: DAY }, TODAY),
		{
			hours: 'Variable',
			tag: null,
			week: null,
		},
	)
	const cycle = rotation(
		[block('Entreprise', FULL, 3), block('CFA', NO_SCHEDULE)],
		'2026-11-02',
	)
	const summary = columnSummary(cycle, TODAY)
	assert.equal(summary.hours, '≈ 30 h')
	assert.equal(summary.tag, 'Rotation')
	assert.deepEqual(summary.week, FULL)
	const school = columnSummary(cycle, '2026-11-25')
	assert.deepEqual(school.week, NO_SCHEDULE)
})

test('previewLines de Yanis montre la semaine de CFA du 23 novembre', () => {
	const yanis = state.members.find((member) => member.id === 'yanis')
	assert.ok(yanis)
	const schedule = yanis.contracts[0].schedule
	assert.equal(schedule.kind, 'rotation')
	if (schedule.kind !== 'rotation') return
	const lines = previewLines(schedule, TODAY, 4)
	assert.equal(lines.length, 4)
	assert.equal(lines[0].current, true)
	assert.equal(lines[1].current, false)
	const school = lines.find((line) => line.label === 'CFA')
	assert.ok(school)
	assert.equal(school.start, '2026-11-23')
	assert.equal(school.end, '2026-11-29')
	assert.equal(school.summary, 'Pas de travail')
})

test('previewLines donne un libellé par défaut aux séquences sans nom', () => {
	const cycle = rotation([block('', FULL, 2), block('', THREE_DAYS, 2)])
	if (cycle.kind !== 'rotation') return
	const lines = previewLines(cycle, '2026-11-02', 3)
	assert.deepEqual(
		lines.map((line) => line.label),
		['Séquence 1', 'Séquence 2', 'Séquence 1'],
	)
	assert.equal(lines[0].summary, '40 h · Lun–Ven')
	assert.equal(lines[0].start, '2026-11-02')
	assert.equal(lines[0].end, '2026-11-15')
})

test('variableMonthSummary compte les jours planifiés de Manon en novembre', () => {
	assert.deepEqual(variableMonthSummary(state, 'manon', '2026-11-04'), {
		days: 13,
		minutes: 13 * 8 * 60,
	})
	assert.deepEqual(variableMonthSummary(state, 'manon', '2026-12-04'), {
		days: 0,
		minutes: 0,
	})
})

test('rhythmLines résume le rythme de chaque contrat', () => {
	assert.deepEqual(rhythmLines(weekly(FULL)), {
		volume: '40 h par semaine',
		detail: 'Lun–Ven',
	})
	assert.deepEqual(
		rhythmLines(
			rotation([block('Entreprise', FULL, 3), block('CFA', NO_SCHEDULE)]),
		),
		{
			volume: '≈ 30 h par semaine en moyenne · rotation de 4 semaines',
			detail: 'Entreprise (3 semaines) puis CFA (1 semaine)',
		},
	)
	assert.deepEqual(rhythmLines({ kind: 'variable', defaultDay: DAY }), {
		volume: 'Planning variable',
		detail: null,
	})
})
