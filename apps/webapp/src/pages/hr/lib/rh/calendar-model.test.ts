import { assert, test } from 'vitest'
import {
	cellLabel,
	cellModel,
	dayDetails,
	extraBadge,
	isSameMonth,
	monthLabel,
	monthWeeks,
	shiftMonth,
} from '#/pages/hr/lib/rh/calendar-model'
import type { Member, RhState } from '#/pages/hr/lib/rh/types'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE

function memberOf(id: string): Member {
	const found = state.members.find((member) => member.id === id)
	if (!found) throw new Error(id)
	return found
}

test('monthWeeks couvre le mois par semaines complètes du lundi au dimanche', () => {
	const weeks = monthWeeks('2026-11-04')
	assert.equal(weeks.length, 6)
	assert.ok(weeks.every((week) => week.length === 7))
	assert.equal(weeks[0][0], '2026-10-26')
	assert.equal(weeks[5][6], '2026-12-06')
	assert.ok(weeks.flat().includes('2026-11-30'))
})

test('monthWeeks compte 5 semaines quand le mois tient dessus', () => {
	const weeks = monthWeeks('2026-02-10')
	assert.equal(weeks.length, 5)
	assert.equal(weeks[0][0], '2026-01-26')
	assert.equal(weeks[4][6], '2026-03-01')
})

test('monthWeeks accepte n’importe quel jour du mois', () => {
	assert.deepEqual(monthWeeks('2026-11-30'), monthWeeks('2026-11-01'))
})

test('shiftMonth avance et recule au premier du mois', () => {
	assert.equal(shiftMonth('2026-11-17', 1), '2026-12-01')
	assert.equal(shiftMonth('2026-11-17', -1), '2026-10-01')
	assert.equal(shiftMonth('2026-12-31', 1), '2027-01-01')
	assert.equal(shiftMonth('2026-01-10', -1), '2025-12-01')
})

test('monthLabel met une majuscule au mois', () => {
	assert.equal(monthLabel('2026-11-04'), 'Novembre 2026')
	assert.equal(monthLabel('2027-02-01'), 'Février 2027')
})

test('isSameMonth compare année et mois', () => {
	assert.equal(isSameMonth('2026-11-01', '2026-11-30'), true)
	assert.equal(isSameMonth('2026-11-01', '2025-11-01'), false)
	assert.equal(isSameMonth('2026-11-30', '2026-12-01'), false)
})

test('un jour travaillé donne la plage compacte et la plage complète', () => {
	const cell = cellModel(state, memberOf('karim'), '2026-11-02')
	assert.equal(cell.split, false)
	assert.equal(cell.compactRange, '8h–17h')
	assert.equal(cell.dayRange, '8h–12h · 13h–17h')
	assert.equal(cell.morning.kind, 'work')
	assert.equal(cell.overridden, false)
	assert.equal(cell.extraApproved, 0)
})

test('un week-end non travaillé est un repos', () => {
	const cell = cellModel(state, memberOf('karim'), '2026-11-08')
	assert.equal(cell.morning.kind, 'off')
	assert.equal(cell.compactRange, null)
})

test('un dimanche férié est signalé avec son nom', () => {
	const cell = cellModel(state, memberOf('karim'), '2026-11-01')
	assert.deepEqual(cell.morning, { kind: 'holiday', name: 'Toussaint' })
	assert.equal(cell.split, false)
})

test('une absence en attente sur deux jours reste pleine journée', () => {
	for (const date of ['2026-11-05', '2026-11-06']) {
		const cell = cellModel(state, memberOf('julien'), date)
		assert.equal(cell.split, false)
		assert.deepEqual(
			{
				kind: cell.morning.kind,
				pending: cell.morning.kind === 'absence' && cell.morning.pending,
				typeId: cell.morning.kind === 'absence' && cell.morning.typeId,
			},
			{ kind: 'absence', pending: true, typeId: 'cp' },
		)
	}
})

test('la demi-journée de Lucie coupe la cellule en deux', () => {
	const cell = cellModel(state, memberOf('lucie'), '2026-11-04')
	assert.equal(cell.split, true)
	assert.deepEqual(cell.morning, { kind: 'work', range: '9h–12h30' })
	assert.equal(cell.afternoon.kind, 'absence')
	assert.equal(
		cell.afternoon.kind === 'absence' && cell.afternoon.pending,
		false,
	)
	assert.equal(cell.compactRange, null)
})

test('un jour avant l’entrée est hors contrat', () => {
	const cell = cellModel(state, memberOf('manon'), '2026-10-05')
	assert.equal(cell.morning.kind, 'inactive')
	assert.equal(cell.afternoon.kind, 'inactive')
})

test('les heures extra validées du 3 novembre de Julien sont totalisées', () => {
	const cell = cellModel(state, memberOf('julien'), '2026-11-03')
	assert.equal(cell.extraApproved, 180)
	assert.equal(cell.extraPending, 0)
	assert.equal(extraBadge(cell.extraApproved), '+3 h')
	assert.equal(extraBadge(150), '+2 h 30')
})

test('les heures extra en attente sont comptées à part', () => {
	const cell = cellModel(state, memberOf('hugo'), '2026-11-14')
	assert.equal(cell.extraApproved, 0)
	assert.equal(cell.extraPending, 240)
})

test('les heures extra refusées ne comptent pas', () => {
	const cell = cellModel(state, memberOf('thomas'), '2026-10-31')
	assert.equal(cell.extraApproved, 0)
	assert.equal(cell.extraPending, 0)
})

test('un jour modifié est repéré avec sa note et ses horaires', () => {
	const override = state.overrides.find(
		(other) => other.memberId === 'karim' && other.date === '2026-11-07',
	)
	assert.ok(override)
	const cell = cellModel(state, memberOf('karim'), '2026-11-07')
	assert.equal(cell.overridden, true)
	assert.equal(cell.note, override.note)
	assert.equal(cell.compactRange, '8h–12h')
	assert.equal(cell.lunch, null)
	assert.equal(
		cellLabel(state, cell),
		'samedi 7 novembre : au travail 8h–12h, jour modifié',
	)
})

test('cellLabel décrit une journée travaillée avec ses heures extra', () => {
	const julien = memberOf('julien')
	assert.equal(
		cellLabel(state, cellModel(state, julien, '2026-11-03')),
		'mardi 3 novembre : au travail 8h–12h · 13h–17h, heures extra : 3 h validées',
	)
})

test('cellLabel décrit une absence en attente', () => {
	const julien = memberOf('julien')
	assert.equal(
		cellLabel(state, cellModel(state, julien, '2026-11-05')),
		'jeudi 5 novembre : Congés payés, à valider',
	)
})

test('cellLabel décrit une demi-journée', () => {
	const lucie = memberOf('lucie')
	assert.equal(
		cellLabel(state, cellModel(state, lucie, '2026-11-04')),
		'mercredi 4 novembre : matin au travail 9h–12h30, après-midi Congés payés',
	)
})

test('cellLabel décrit repos, férié et hors contrat', () => {
	assert.equal(
		cellLabel(state, cellModel(state, memberOf('karim'), '2026-11-08')),
		'dimanche 8 novembre : repos',
	)
	assert.equal(
		cellLabel(state, cellModel(state, memberOf('karim'), '2026-11-01')),
		'dimanche 1 novembre : férié (Toussaint)',
	)
	assert.equal(
		cellLabel(state, cellModel(state, memberOf('manon'), '2026-10-05')),
		'lundi 5 octobre : hors contrat',
	)
})

test('cellLabel mentionne les heures extra en attente et le jour modifié', () => {
	const hugo = memberOf('hugo')
	assert.equal(
		cellLabel(state, cellModel(state, hugo, '2026-11-14')),
		'samedi 14 novembre : repos, heures extra : 4 h en attente',
	)
	const label = cellLabel(
		state,
		cellModel(state, memberOf('karim'), '2026-11-07'),
	)
	assert.ok(label.endsWith(', jour modifié'))
})

test('dayDetails garde une seule ligne pour une journée entière', () => {
	const details = dayDetails(state, memberOf('karim'), '2026-11-02')
	assert.equal(details.halves.length, 1)
	assert.equal(details.halves[0].part, 'Journée')
	assert.equal(details.lunch, '12h–13h')
	assert.equal(details.dayRange, '8h–12h · 13h–17h')
	assert.equal(details.title, 'lundi 2 novembre')
})

test('dayDetails sépare matin et après-midi et retrouve l’absence', () => {
	const details = dayDetails(state, memberOf('lucie'), '2026-11-04')
	assert.deepEqual(
		details.halves.map((half) => half.part),
		['Matin', 'Après-midi'],
	)
	assert.equal(details.halves[0].absence, null)
	assert.equal(details.halves[1].absence?.id, 'abs-lucie-aprem')
	assert.equal(details.lunch, '12h30–13h30')
})

test('dayDetails liste les heures extra actives du jour', () => {
	const details = dayDetails(state, memberOf('julien'), '2026-11-03')
	assert.deepEqual(
		details.extras.map((extra) => extra.id),
		['extra-julien-soir'],
	)
})

test('dayDetails n’a pas de pause un jour de repos', () => {
	const details = dayDetails(state, memberOf('karim'), '2026-11-08')
	assert.equal(details.lunch, null)
})

test('une absence annulée ne colore pas la cellule', () => {
	const cancelled: RhState = {
		...state,
		absences: state.absences.map((absence) =>
			absence.memberId === 'julien' && absence.startDate === '2026-11-05'
				? { ...absence, status: 'cancelled' }
				: absence,
		),
	}
	const cell = cellModel(cancelled, memberOf('julien'), '2026-11-05')
	assert.equal(cell.morning.kind, 'work')
})

test('la semaine de CFA de Yanis remplace « repos » par le libellé', () => {
	const yanis = memberOf('yanis')
	const cell = cellModel(state, yanis, '2026-11-23')
	assert.deepEqual(cell.morning, { kind: 'off', label: 'CFA' })
	assert.equal(cell.split, false)
	assert.equal(cellLabel(state, cell), 'lundi 23 novembre : CFA')
	const weekend = cellModel(state, yanis, '2026-11-28')
	assert.deepEqual(weekend.morning, { kind: 'off', label: 'CFA' })
	assert.equal(cellModel(state, yanis, '2026-11-14').morning.kind, 'off')
	assert.equal(
		cellLabel(state, cellModel(state, yanis, '2026-11-14')),
		'samedi 14 novembre : repos',
	)
})

test('la rotation de Yanis travaille les semaines d’entreprise', () => {
	const cell = cellModel(state, memberOf('yanis'), '2026-11-04')
	assert.equal(cell.morning.kind, 'work')
	assert.equal(cell.compactRange, '8h–16h')
})

test('la semaine B de Hugo laisse le vendredi 13 novembre en repos', () => {
	const hugo = memberOf('hugo')
	assert.deepEqual(cellModel(state, hugo, '2026-11-13').morning, {
		kind: 'off',
	})
	assert.equal(cellModel(state, hugo, '2026-11-06').morning.kind, 'work')
	assert.equal(
		cellLabel(state, cellModel(state, hugo, '2026-11-13')),
		'vendredi 13 novembre : repos',
	)
})

test('un jour non planifié d’un contrat variable est à planifier', () => {
	const manon = memberOf('manon')
	const cell = cellModel(state, manon, '2026-11-24')
	assert.deepEqual(cell.morning, { kind: 'unplanned' })
	assert.equal(cell.variable, true)
	assert.equal(cell.planned, false)
	assert.equal(cell.overridden, false)
	assert.equal(cellLabel(state, cell), 'mardi 24 novembre : à planifier')
})

test('un jour planifié d’un contrat variable n’est pas un jour modifié', () => {
	const manon = memberOf('manon')
	const worked = cellModel(state, manon, '2026-11-04')
	assert.equal(worked.morning.kind, 'work')
	assert.equal(worked.planned, true)
	assert.equal(worked.overridden, false)
	assert.equal(
		cellLabel(state, worked),
		'mercredi 4 novembre : au travail 8h–12h · 13h–17h',
	)
	const off = cellModel(state, manon, '2026-11-13')
	assert.equal(off.morning.kind, 'off')
	assert.equal(off.planned, true)
	assert.equal(cellLabel(state, off), 'vendredi 13 novembre : repos')
})

test('un jour férié d’un contrat variable reste férié', () => {
	assert.equal(
		cellModel(state, memberOf('manon'), '2026-11-11').morning.kind,
		'holiday',
	)
})

test('dayDetails propose la planification pour un contrat variable', () => {
	const manon = memberOf('manon')
	const free = dayDetails(state, manon, '2026-11-24')
	assert.equal(free.variable?.planned, false)
	assert.equal(free.variable?.defaultDay.start, '08:00')
	assert.equal(free.halves[0].model.kind, 'unplanned')
	const planned = dayDetails(state, manon, '2026-11-04')
	assert.equal(planned.variable?.planned, true)
	assert.equal(planned.note, null)
	assert.equal(
		dayDetails(state, memberOf('karim'), '2026-11-02').variable,
		null,
	)
})

test('dayDetails donne le libellé de la séquence d’une rotation', () => {
	assert.equal(
		dayDetails(state, memberOf('yanis'), '2026-11-04').sequence,
		'Entreprise',
	)
	assert.equal(
		dayDetails(state, memberOf('yanis'), '2026-11-23').sequence,
		'CFA',
	)
	assert.equal(
		dayDetails(state, memberOf('hugo'), '2026-11-13').sequence,
		'Semaine B',
	)
	assert.equal(
		dayDetails(state, memberOf('karim'), '2026-11-04').sequence,
		null,
	)
})
