import { assert, test } from 'vitest'
import {
	contractCountLabel,
	contractPeriod,
	contractTerm,
	dayHoursLabel,
	dueLabel,
	durationLabel,
	formatDate,
	scheduleCaption,
	tenureLabel,
	workedDaysLabel,
} from '#/pages/hr/lib/rh/format'
import { defaultSchedule } from '#/pages/hr/lib/rh/members'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import type { WeekSchedule, WorkDay } from '#/pages/hr/lib/rh/types'

const DAY: WorkDay = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }

function week(...worked: number[]): WeekSchedule {
	return Array.from({ length: 7 }, (_, index) =>
		worked.includes(index) ? DAY : null,
	)
}

test('dayHoursLabel donne la durée nette de la pause', () => {
	assert.equal(
		dayHoursLabel({ start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }),
		'8 h',
	)
	assert.equal(
		dayHoursLabel({
			start: '09:00',
			end: '17:30',
			lunch: { start: '12:30', end: '13:30' },
		}),
		'7,5 h',
	)
	assert.equal(
		dayHoursLabel({ start: '08:00', end: '17:00', lunch: null }),
		'9 h',
	)
	assert.equal(dayHoursLabel({ start: '08:00', end: '12:00' }), '4 h')
})

test('dueLabel exprime le délai en jours', () => {
	assert.equal(dueLabel(0), 'aujourd’hui')
	assert.equal(dueLabel(1), 'demain')
	assert.equal(dueLabel(2), 'dans 2 jours')
	assert.equal(dueLabel(41), 'dans 41 jours')
})

test('formatDate affiche le jour, le mois abrégé et l’année', () => {
	assert.equal(formatDate('2019-09-02'), '2 sept. 2019')
	assert.equal(formatDate('2026-12-18'), '18 déc. 2026')
})

test('formatDate écrit 1er pour le premier jour du mois seulement', () => {
	assert.equal(formatDate('2021-09-01'), '1er sept. 2021')
	assert.equal(formatDate('2022-03-11'), '11 mars 2022')
	assert.equal(formatDate('2022-03-21'), '21 mars 2022')
})

test('tenureLabel distingue moins d’un an, un an et plusieurs années', () => {
	assert.equal(tenureLabel(0), 'moins d’un an')
	assert.equal(tenureLabel(1), '1 an')
	assert.equal(tenureLabel(7), '7 ans')
})

test('contractPeriod décrit une période fermée ou ouverte', () => {
	assert.equal(
		contractPeriod({ startDate: '2021-09-01', endDate: '2022-02-28' }),
		'1er sept. 2021 au 28 févr. 2022',
	)
	assert.equal(
		contractPeriod({ startDate: '2022-03-01', endDate: null }),
		'depuis le 1er mars 2022',
	)
})

test('contractPeriod annonce un contrat ouvert pas encore commencé', () => {
	assert.equal(
		contractPeriod({ startDate: '2026-12-19', endDate: null }, '2026-11-04'),
		'à partir du 19 déc. 2026',
	)
	assert.equal(
		contractPeriod({ startDate: '2022-03-01', endDate: null }, '2026-11-04'),
		'depuis le 1er mars 2022',
	)
})

test('contractTerm dépend de la phase du contrat', () => {
	const today = '2026-11-04'
	assert.equal(
		contractTerm({ startDate: '2022-03-01', endDate: null }, today),
		'Depuis le 1er mars 2022',
	)
	assert.equal(
		contractTerm({ startDate: '2026-05-04', endDate: '2026-12-18' }, today),
		'Jusqu’au 18 déc. 2026',
	)
	assert.equal(
		contractTerm({ startDate: '2026-12-19', endDate: null }, today),
		'À partir du 19 déc. 2026',
	)
	assert.equal(
		contractTerm({ startDate: '2025-01-06', endDate: '2025-06-30' }, today),
		'Terminé le 30 juin 2025',
	)
})

test('durationLabel exprime la durée en jours, mois ou années', () => {
	const today = '2026-11-04'
	assert.equal(durationLabel('2026-10-12', '2026-11-27', today), '47 jours')
	assert.equal(durationLabel('2026-11-04', '2026-11-04', today), '1 jour')
	assert.equal(durationLabel('2021-09-01', '2022-02-28', today), '6 mois')
	assert.equal(durationLabel('2019-09-02', '2021-08-31', today), '2 ans')
	assert.equal(
		durationLabel('2021-01-04', '2023-08-31', today),
		'2 ans et 8 mois',
	)
	assert.equal(durationLabel('2025-09-01', '2026-08-31', today), '1 an')
})

test('durationLabel d’un contrat ouvert court jusqu’à aujourd’hui', () => {
	assert.equal(
		durationLabel('2022-03-01', null, '2026-11-04'),
		'4 ans et 8 mois',
	)
	assert.equal(durationLabel('2026-07-06', null, '2026-11-04'), '4 mois')
})

test('durationLabel est nul pour un contrat ouvert pas encore commencé', () => {
	assert.equal(durationLabel('2026-12-19', null, '2026-11-04'), null)
})

test('workedDaysLabel regroupe les jours consécutifs', () => {
	assert.equal(workedDaysLabel(week(0, 1, 2, 3, 4)), 'Lun–Ven')
	assert.equal(workedDaysLabel(week(0, 1, 2, 3)), 'Lun–Jeu')
	assert.equal(workedDaysLabel(week(0, 2, 4)), 'Lun, Mer, Ven')
	assert.equal(workedDaysLabel(week(0, 1, 3, 4, 5)), 'Lun, Mar, Jeu–Sam')
	assert.equal(workedDaysLabel(week(2)), 'Mer')
	assert.equal(workedDaysLabel(week()), 'Aucun jour travaillé')
	assert.equal(workedDaysLabel(defaultSchedule()), 'Lun–Ven')
})

test('contractCountLabel pluralise', () => {
	assert.equal(contractCountLabel(1), '1 contrat')
	assert.equal(contractCountLabel(3), '3 contrats')
})

test('scheduleCaption précise la phase du contrat affiché', () => {
	const today = '2026-11-04'
	assert.equal(
		scheduleCaption(
			{ type: 'cdi', startDate: '2022-03-01', endDate: null },
			today,
		),
		'Contrat en cours : CDI, depuis le 1er mars 2022',
	)
	assert.equal(
		scheduleCaption(
			{ type: 'cdi', startDate: '2026-12-19', endDate: null },
			today,
		),
		'Contrat à venir : CDI, à partir du 19 déc. 2026',
	)
	assert.equal(
		scheduleCaption(
			{ type: 'cdd', startDate: '2025-01-06', endDate: '2025-06-30' },
			today,
		),
		'Contrat terminé : CDD, 6 janv. 2025 au 30 juin 2025',
	)
})
