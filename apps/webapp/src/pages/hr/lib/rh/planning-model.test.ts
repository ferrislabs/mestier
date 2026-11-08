import { assert, test } from 'vitest'
import { extrasOn } from '#/pages/hr/lib/rh/extra'
import {
	activePlanningMembers,
	presenceOn,
	resolveDay,
} from '#/pages/hr/lib/rh/planning'
import {
	cellLabel,
	dateOnViewChange,
	dayFormError,
	dayFormFault,
	dayLines,
	describeExtras,
	describeHalf,
	extraLine,
	extraTotals,
	filterMembers,
	initialDayForm,
	isDayFormDirty,
	lineTitle,
	overrideNote,
	periodDays,
	periodLabel,
	presenceSummary,
	presentCount,
	restExtraLabel,
	sameHalf,
	shiftPeriod,
	shortWork,
	toOverride,
	tooltipLines,
	typeIndex,
} from '#/pages/hr/lib/rh/planning-model'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const types = typeIndex(state.absenceTypes)
const member = (id: string) => {
	const found = state.members.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}

test('periodDays renvoie la semaine du lundi au dimanche ou le mois entier', () => {
	const week = periodDays('week', '2026-11-04')
	assert.equal(week.length, 7)
	assert.equal(week[0], '2026-11-02')
	assert.equal(week[6], '2026-11-08')
	const month = periodDays('month', '2026-11-17')
	assert.equal(month.length, 30)
	assert.equal(month[0], '2026-11-01')
	assert.equal(month[29], '2026-11-30')
})

test('shiftPeriod avance de 7 jours ou d’un mois', () => {
	assert.equal(shiftPeriod('week', '2026-11-04', 1), '2026-11-11')
	assert.equal(shiftPeriod('week', '2026-11-04', -1), '2026-10-28')
	assert.equal(shiftPeriod('month', '2026-11-04', 1), '2026-12-01')
	assert.equal(shiftPeriod('month', '2026-11-04', -1), '2026-10-01')
	assert.equal(shiftPeriod('month', '2026-12-15', 1), '2027-01-01')
})

test('periodLabel couvre une semaine, un chevauchement de mois ou d’année et un mois', () => {
	assert.equal(periodLabel('week', '2026-11-04'), '2 – 8 novembre 2026')
	assert.equal(
		periodLabel('week', '2026-10-28'),
		'26 octobre – 1 novembre 2026',
	)
	assert.equal(
		periodLabel('week', '2026-12-30'),
		'28 décembre 2026 – 3 janvier 2027',
	)
	assert.equal(periodLabel('month', '2026-11-17'), 'novembre 2026')
})

test('dateOnViewChange revient à aujourd’hui si le mois affiché le contient', () => {
	assert.equal(
		dateOnViewChange('week', '2026-11-01', '2026-11-04'),
		'2026-11-04',
	)
	assert.equal(
		dateOnViewChange('week', '2026-12-01', '2026-11-04'),
		'2026-12-01',
	)
	assert.equal(
		dateOnViewChange('month', '2026-11-09', '2026-11-04'),
		'2026-11-09',
	)
})

test('filterMembers ignore la casse et les accents et cherche aussi le poste', () => {
	const members = activePlanningMembers(state)
	assert.deepEqual(
		filterMembers(members, 'JUL').map((other) => other.id),
		['julien'],
	)
	assert.deepEqual(
		filterMembers(members, 'electricien').map((other) => other.id),
		['thomas'],
	)
	assert.deepEqual(
		filterMembers(members, 'karim chef').map((other) => other.id),
		['karim'],
	)
	assert.equal(filterMembers(members, '  ').length, members.length)
	assert.equal(filterMembers(members, 'zzz').length, 0)
})

test('sameHalf compare le contenu utile de deux demi-journées', () => {
	const lucie = resolveDay(state, member('lucie'), '2026-11-04')
	assert.equal(sameHalf(lucie.morning, lucie.afternoon), false)
	const thomas = resolveDay(state, member('thomas'), '2026-11-04')
	assert.equal(sameHalf(thomas.morning, thomas.afternoon), true)
	const julien = resolveDay(state, member('julien'), '2026-11-05')
	assert.equal(sameHalf(julien.morning, julien.afternoon), true)
})

test('describeHalf formule chaque état en français', () => {
	const julien = member('julien')
	const day = resolveDay(state, julien, '2026-11-05')
	assert.equal(
		describeHalf(day.morning, 'day', types, julien, day.date),
		'congés payés, à valider',
	)
	const thomas = member('thomas')
	const work = resolveDay(state, thomas, '2026-11-04')
	assert.equal(
		describeHalf(work.morning, 'day', types, thomas, work.date),
		'travaille 8h–12h · 13h–17h',
	)
	assert.equal(
		describeHalf(work.morning, 'morning', types, thomas, work.date),
		'travaille 8h–12h',
	)
	assert.equal(
		describeHalf(work.afternoon, 'afternoon', types, thomas, work.date),
		'travaille 13h–17h',
	)
	const rest = resolveDay(state, thomas, '2026-11-07')
	assert.equal(
		describeHalf(rest.morning, 'day', types, thomas, rest.date),
		'repos',
	)
	const holiday = resolveDay(state, thomas, '2026-11-11')
	assert.equal(
		describeHalf(holiday.morning, 'day', types, thomas, holiday.date),
		'jour férié, Armistice 1918',
	)
	const manon = member('manon')
	const before = resolveDay(state, manon, '2026-10-05')
	assert.equal(
		describeHalf(before.morning, 'day', types, manon, before.date),
		'pas encore dans l’équipe',
	)
	const after = resolveDay(state, manon, '2026-12-01')
	assert.equal(
		describeHalf(after.morning, 'day', types, manon, after.date),
		'contrat terminé',
	)
})

test('describeHalf garde les sigles en majuscules', () => {
	const thomas = member('thomas')
	const day = resolveDay(state, thomas, '2026-11-20')
	assert.equal(describeHalf(day.morning, 'day', types, thomas, day.date), 'RTT')
})

test('cellLabel décrit la cellule pour les lecteurs d’écran', () => {
	const julien = member('julien')
	assert.equal(
		cellLabel(resolveDay(state, julien, '2026-11-05'), types, julien),
		'Julien Roche, jeudi 5 novembre : congés payés, à valider',
	)
	const lucie = member('lucie')
	assert.equal(
		cellLabel(resolveDay(state, lucie, '2026-11-04'), types, lucie),
		'Lucie Martin, mercredi 4 novembre : matin, travaille 9h–12h30 ; après-midi, congés payés',
	)
	const karim = member('karim')
	assert.equal(
		cellLabel(resolveDay(state, karim, '2026-11-07'), types, karim),
		'Karim Benali, samedi 7 novembre : travaille 8h–12h, jour modifié',
	)
})

test('dayLines sépare matin et après-midi avec les heures réelles', () => {
	const thomas = member('thomas')
	assert.deepEqual(
		dayLines(resolveDay(state, thomas, '2026-11-04'), types, thomas),
		[
			{ label: 'Matin', hours: null, text: '8h–12h' },
			{ label: 'Après-midi', hours: null, text: '13h–17h' },
		],
	)
	const lucie = member('lucie')
	assert.deepEqual(
		dayLines(resolveDay(state, lucie, '2026-11-04'), types, lucie),
		[
			{ label: 'Matin', hours: null, text: '9h–12h30' },
			{ label: 'Après-midi', hours: '13h30–17h30', text: 'Congés payés' },
		],
	)
	const julien = member('julien')
	assert.deepEqual(
		dayLines(resolveDay(state, julien, '2026-11-05'), types, julien),
		[{ label: 'Journée', hours: null, text: 'Congés payés, à valider' }],
	)
})

test('une moitié sans heures travaillées est signalée', () => {
	const karim = member('karim')
	const day = resolveDay(state, karim, '2026-11-07')
	assert.deepEqual(dayLines(day, types, karim), [
		{ label: 'Matin', hours: null, text: '8h–12h' },
		{ label: 'Après-midi', hours: null, text: 'Pas de travail' },
	])
	assert.equal(shortWork(day.afternoon, 'afternoon'), '')
	assert.equal(
		describeHalf(day.afternoon, 'afternoon', types, karim, day.date),
		'pas de travail',
	)
})

test('tooltipLines condense une journée travaillée avec sa pause', () => {
	const thomas = member('thomas')
	assert.deepEqual(
		tooltipLines(resolveDay(state, thomas, '2026-11-04'), types, thomas),
		[{ label: 'Journée', hours: null, text: 'Travaille 8h–12h · 13h–17h' }],
	)
	const lucie = member('lucie')
	const lines = tooltipLines(
		resolveDay(state, lucie, '2026-11-04'),
		types,
		lucie,
	)
	assert.equal(lines.length, 2)
	assert.equal(lineTitle(lines[1]), 'Après-midi (13h30–17h30)')
	assert.equal(lineTitle(lines[0]), 'Matin')
})

test('shortWork garde la cellule compacte et donne les heures réelles des moitiés', () => {
	const thomas = member('thomas')
	const day = resolveDay(state, thomas, '2026-11-04')
	assert.equal(shortWork(day.morning, 'day'), '8h–17h')
	assert.equal(shortWork(day.morning, 'morning'), '8h–12h')
	assert.equal(shortWork(day.afternoon, 'afternoon'), '13h–17h')
	assert.equal(shortWork({ kind: 'inactive' }, 'day'), '')
	const lucie = member('lucie')
	const split = resolveDay(state, lucie, '2026-11-04')
	assert.equal(shortWork(split.morning, 'morning'), '9h–12h30')
})

test('presentCount compte les membres qui travaillent au moins une demi-journée', () => {
	const members = activePlanningMembers(state)
	assert.equal(presentCount(state, members, '2026-11-04'), 8)
	assert.equal(presentCount(state, members, '2026-11-07'), 1)
	assert.equal(presentCount(state, members, '2026-11-11'), 0)
	assert.equal(presentCount(state, [], '2026-11-04'), 0)
})

test('presenceSummary accorde le pluriel', () => {
	assert.equal(
		presenceSummary(presenceOn(state, '2026-11-04')),
		'7 au travail · 2 absents · 0 en repos',
	)
	assert.equal(
		presenceSummary(presenceOn(state, '2026-11-09')),
		'8 au travail · 1 absent · 0 en repos',
	)
})

test('initialDayForm part de la semaine type, pause comprise, ou du forçage existant', () => {
	const lucie = member('lucie')
	assert.deepEqual(initialDayForm(lucie, '2026-11-05', null), {
		kind: 'work',
		start: '09:00',
		end: '17:30',
		lunchOn: true,
		lunchStart: '12:30',
		lunchEnd: '13:30',
		note: '',
	})
	assert.deepEqual(initialDayForm(lucie, '2026-11-06', null), {
		kind: 'off',
		start: '08:00',
		end: '17:00',
		lunchOn: true,
		lunchStart: '12:00',
		lunchEnd: '13:00',
		note: '',
	})
	assert.equal(initialDayForm(lucie, '2026-11-11', null).kind, 'off')
	const override = {
		memberId: 'karim',
		date: '2026-11-07',
		kind: 'work' as const,
		start: '08:00',
		end: '12:00',
		lunch: null,
		note: 'Livraison',
	}
	assert.deepEqual(initialDayForm(member('karim'), '2026-11-07', override), {
		kind: 'work',
		start: '08:00',
		end: '12:00',
		lunchOn: false,
		lunchStart: '12:00',
		lunchEnd: '13:00',
		note: 'Livraison',
	})
	const withLunch = {
		...override,
		start: '08:00',
		end: '16:00',
		lunch: { start: '12:00', end: '13:30' },
	}
	const form = initialDayForm(member('karim'), '2026-11-07', withLunch)
	assert.equal(form.lunchOn, true)
	assert.equal(form.lunchEnd, '13:30')
	const rest = { ...override, kind: 'off' as const, start: '', end: '' }
	assert.deepEqual(initialDayForm(member('julien'), '2026-11-23', rest), {
		kind: 'off',
		start: '08:00',
		end: '17:00',
		lunchOn: true,
		lunchStart: '12:00',
		lunchEnd: '13:00',
		note: 'Livraison',
	})
})

test('initialDayForm n’impose pas de pause si la semaine type n’en a pas', () => {
	const base = member('thomas')
	const current = base.contracts[base.contracts.length - 1]
	const day = { start: '08:00', end: '17:00', lunch: null }
	const continuous = {
		...base,
		contracts: [
			{
				...current,
				schedule: {
					kind: 'weekly' as const,
					week: [day, day, day, day, day, null, null],
				},
			},
		],
	}
	const form = initialDayForm(continuous, '2026-11-04', null)
	assert.equal(form.lunchOn, false)
	assert.equal(form.lunchStart, '12:00')
})

const workForm = {
	kind: 'work' as const,
	start: '08:00',
	end: '17:00',
	lunchOn: true,
	lunchStart: '12:00',
	lunchEnd: '13:00',
	note: '',
}

test('dayFormError valide la journée et la pause avec validateWorkDay', () => {
	assert.equal(dayFormError(workForm), null)
	assert.equal(
		dayFormError({ ...workForm, end: '07:00' }),
		'La fin de journée doit suivre le début.',
	)
	assert.equal(
		dayFormError({ ...workForm, start: '' }),
		'Renseigne l’heure de début et l’heure de fin.',
	)
	assert.equal(
		dayFormError({ ...workForm, lunchEnd: '11:00' }),
		'La fin de la pause doit suivre son début.',
	)
	assert.equal(
		dayFormError({ ...workForm, lunchStart: '07:00', lunchEnd: '09:00' }),
		'La pause doit se situer entre le début et la fin de la journée.',
	)
	assert.equal(
		dayFormError({ ...workForm, lunchEnd: '11:00', lunchOn: false }),
		null,
	)
	assert.equal(dayFormError({ ...workForm, kind: 'off', end: '' }), null)
})

test('dayFormFault désigne la partie fautive du formulaire', () => {
	assert.equal(dayFormFault(workForm), null)
	assert.equal(dayFormFault({ ...workForm, end: '07:00' }), 'day')
	assert.equal(dayFormFault({ ...workForm, lunchEnd: '11:00' }), 'lunch')
	assert.equal(dayFormFault({ ...workForm, kind: 'off', end: '' }), null)
})

test('isDayFormDirty ignore les horaires d’un repos, les espaces de la note et une pause désactivée', () => {
	const initial = { ...workForm, kind: 'off' as const }
	assert.equal(isDayFormDirty(initial, { ...initial, start: '09:00' }), false)
	assert.equal(isDayFormDirty(initial, { ...initial, lunchOn: false }), false)
	assert.equal(isDayFormDirty(initial, { ...initial, note: '  ' }), false)
	assert.equal(isDayFormDirty(initial, { ...initial, note: 'Pont' }), true)
	assert.equal(isDayFormDirty(initial, { ...initial, kind: 'work' }), true)
	assert.equal(isDayFormDirty(workForm, { ...workForm, end: '16:00' }), true)
	assert.equal(isDayFormDirty(workForm, { ...workForm, lunchOn: false }), true)
	assert.equal(
		isDayFormDirty(workForm, { ...workForm, lunchEnd: '13:30' }),
		true,
	)
	const noLunch = { ...workForm, lunchOn: false }
	assert.equal(
		isDayFormDirty(noLunch, { ...noLunch, lunchEnd: '13:30' }),
		false,
	)
})

test('toOverride vide les horaires d’un repos, porte la pause et nettoie la note', () => {
	assert.deepEqual(
		toOverride('julien', '2026-11-23', {
			...workForm,
			kind: 'off',
			note: '  Pont accordé ',
		}),
		{
			memberId: 'julien',
			date: '2026-11-23',
			kind: 'off',
			start: '',
			end: '',
			lunch: null,
			note: 'Pont accordé',
		},
	)
	assert.deepEqual(
		toOverride('karim', '2026-11-07', {
			...workForm,
			end: '16:00',
			lunchEnd: '13:30',
		}),
		{
			memberId: 'karim',
			date: '2026-11-07',
			kind: 'work',
			start: '08:00',
			end: '16:00',
			lunch: { start: '12:00', end: '13:30' },
			note: '',
		},
	)
	assert.equal(
		toOverride('karim', '2026-11-07', { ...workForm, lunchOn: false }).lunch,
		null,
	)
})

const extrasFor = (id: string, date: string) => extrasOn(state, id, date)

test('extraTotals cumule les minutes nettes validées et en attente', () => {
	assert.deepEqual(extraTotals(extrasFor('julien', '2026-11-03')), {
		approved: 180,
		pending: 0,
	})
	assert.deepEqual(extraTotals(extrasFor('lucie', '2026-11-02')), {
		approved: 90,
		pending: 0,
	})
	assert.deepEqual(extraTotals(extrasFor('sophie', '2026-11-11')), {
		approved: 0,
		pending: 240,
	})
	assert.deepEqual(extraTotals([]), { approved: 0, pending: 0 })
	const both = [
		...extrasFor('julien', '2026-11-03'),
		...extrasFor('sophie', '2026-11-11'),
	]
	assert.deepEqual(extraTotals(both), { approved: 180, pending: 240 })
})

test('extraLine résume plage, durée, majoration et statut', () => {
	const [julien] = extrasFor('julien', '2026-11-03')
	assert.equal(extraLine(julien), '17h–20h · 3 h · +25 %')
	const [lucie] = extrasFor('lucie', '2026-11-02')
	assert.equal(extraLine(lucie), '17h30–19h · 1 h 30 · Sans majoration')
	const [sophie] = extrasFor('sophie', '2026-11-11')
	assert.equal(extraLine(sophie), '8h–12h · 4 h · +100 % · à valider')
})

test('describeExtras prolonge l’aria-label avec les heures extra', () => {
	assert.equal(describeExtras([]), '')
	assert.equal(
		describeExtras(extrasFor('julien', '2026-11-03')),
		' ; heures extra 17h–20h, majorées de 25 %',
	)
	assert.equal(
		describeExtras(extrasFor('lucie', '2026-11-02')),
		' ; heures extra 17h30–19h',
	)
	assert.equal(
		describeExtras(extrasFor('sophie', '2026-11-11')),
		' ; heures extra 8h–12h, majorées de 100 %, à valider',
	)
})

test('cellLabel ajoute les heures extra du jour', () => {
	const julien = member('julien')
	assert.equal(
		cellLabel(
			resolveDay(state, julien, '2026-11-03'),
			types,
			julien,
			extrasFor('julien', '2026-11-03'),
		),
		'Julien Roche, mardi 3 novembre : travaille 8h–12h · 13h–17h ; heures extra 17h–20h, majorées de 25 %',
	)
	const hugo = member('hugo')
	assert.equal(
		cellLabel(
			resolveDay(state, hugo, '2026-10-25'),
			types,
			hugo,
			extrasFor('hugo', '2026-10-25'),
		),
		'Hugo Faure, dimanche 25 octobre : repos ; heures extra 8h–14h, majorées de 100 %',
	)
})

test('restExtraLabel nomme la plage des seules extras validées', () => {
	assert.equal(restExtraLabel(extrasFor('hugo', '2026-10-25')), 'Extra 8h–14h')
	assert.equal(restExtraLabel(extrasFor('hugo', '2026-11-14')), null)
	assert.equal(restExtraLabel([]), null)
})

test('presentCount compte une extra validée un jour de repos comme présence', () => {
	const members = activePlanningMembers(state)
	assert.equal(presentCount(state, members, '2026-10-25'), 1)
	assert.equal(
		presentCount(state, members, '2026-10-25'),
		presenceOn(state, '2026-10-25').working.length,
	)
	assert.equal(presentCount(state, members, '2026-11-14'), 0)
	assert.equal(presentCount(state, members, '2026-11-11'), 0)
	assert.equal(presentCount(state, members, '2026-11-03'), 7)
})

test('presentCount ignore une extra validée si le membre est absent ce jour-là', () => {
	const sophie = member('sophie')
	const sick = state.absences.find((other) => other.id === 'abs-sophie-maladie')
	const extra = state.extras.find((other) => other.id === 'extra-hugo-dimanche')
	if (!sick || !extra) throw new Error('seed')
	const next = {
		...state,
		absences: [
			...state.absences,
			{
				...sick,
				id: 'abs-test',
				startDate: '2026-11-07',
				endDate: '2026-11-07',
			},
		],
		extras: [
			...state.extras,
			{ ...extra, id: 'extra-test', memberId: sophie.id, date: '2026-11-07' },
		],
	}
	assert.equal(presentCount(next, activePlanningMembers(next), '2026-11-07'), 1)
})

test('describeHalf nomme un jour à planifier et le libellé d’une séquence sans travail', () => {
	const manon = member('manon')
	const open = resolveDay(state, manon, '2026-11-25')
	assert.equal(open.variable, true)
	assert.equal(
		describeHalf(open.morning, 'day', types, manon, open.date),
		'à planifier',
	)
	const yanis = member('yanis')
	const cfa = resolveDay(state, yanis, '2026-11-25')
	assert.equal(describeHalf(cfa.morning, 'day', types, yanis, cfa.date), 'CFA')
	const hugo = member('hugo')
	const friday = resolveDay(state, hugo, '2026-11-13')
	assert.equal(
		describeHalf(friday.morning, 'day', types, hugo, friday.date),
		'repos',
	)
})

test('sameHalf distingue un jour à planifier, un libellé de séquence et un repos', () => {
	const manon = member('manon')
	const open = resolveDay(state, manon, '2026-11-25')
	const planned = resolveDay(state, manon, '2026-11-13')
	assert.equal(sameHalf(open.morning, open.afternoon), true)
	assert.equal(sameHalf(open.morning, planned.morning), false)
	const yanis = member('yanis')
	const cfa = resolveDay(state, yanis, '2026-11-25')
	assert.equal(sameHalf(cfa.morning, { kind: 'off', overridden: false }), false)
	assert.equal(
		sameHalf(
			{ kind: 'off', overridden: false, label: 'CFA' },
			{ kind: 'off', overridden: false, label: 'CFA' },
		),
		true,
	)
})

test('cellLabel et dayLines couvrent les jours à planifier et les séquences', () => {
	const manon = member('manon')
	const open = resolveDay(state, manon, '2026-11-25')
	assert.equal(
		cellLabel(open, types, manon),
		'Manon Petit, mercredi 25 novembre : à planifier',
	)
	assert.deepEqual(dayLines(open, types, manon), [
		{ label: 'Journée', hours: null, text: 'À planifier' },
	])
	const yanis = member('yanis')
	const cfa = resolveDay(state, yanis, '2026-11-25')
	assert.equal(
		cellLabel(cfa, types, yanis),
		'Yanis Moreau, mercredi 25 novembre : CFA',
	)
	assert.deepEqual(dayLines(cfa, types, yanis), [
		{ label: 'Journée', hours: null, text: 'CFA' },
	])
})

test('un jour planifié d’un contrat variable n’est pas un jour modifié', () => {
	const manon = member('manon')
	const planned = resolveDay(state, manon, '2026-11-04')
	assert.equal(planned.override !== null, true)
	assert.equal(
		cellLabel(planned, types, manon),
		'Manon Petit, mercredi 4 novembre : travaille 8h–12h · 13h–17h',
	)
	assert.deepEqual(dayLines(planned, types, manon), [
		{ label: 'Matin', hours: null, text: '8h–12h' },
		{ label: 'Après-midi', hours: null, text: '13h–17h' },
	])
	assert.equal(overrideNote(planned), null)
	const karim = member('karim')
	assert.equal(
		overrideNote(resolveDay(state, karim, '2026-11-07')),
		'Jour modifié : Livraison du matériel sur le chantier Dupont',
	)
	assert.equal(
		overrideNote(resolveDay(state, member('julien'), '2026-11-23')),
		'Jour modifié : Pont accordé',
	)
	assert.equal(overrideNote(resolveDay(state, karim, '2026-11-04')), null)
	const rest = resolveDay(state, manon, '2026-11-13')
	assert.equal(overrideNote(rest), 'Note : Pas de mission')
	assert.equal(
		cellLabel(rest, types, manon),
		'Manon Petit, vendredi 13 novembre : repos',
	)
})

test('presenceSummary ajoute les jours à planifier seulement quand il y en a', () => {
	assert.equal(
		presenceSummary(presenceOn(state, '2026-11-25')),
		'7 au travail · 0 absent · 1 en repos · 1 à planifier',
	)
	assert.equal(
		presenceSummary(presenceOn(state, '2026-11-13')),
		'5 au travail · 1 absent · 3 en repos',
	)
})

test('presentCount ne compte ni les jours à planifier ni une séquence sans travail', () => {
	const members = activePlanningMembers(state)
	assert.equal(presentCount(state, members, '2026-11-25'), 7)
	assert.equal(
		presentCount(state, members, '2026-11-25'),
		presenceOn(state, '2026-11-25').working.length,
	)
	assert.equal(presentCount(state, members, '2026-11-13'), 5)
})

test('initialDayForm d’un contrat variable part du jour par défaut du contrat', () => {
	const manon = member('manon')
	assert.deepEqual(initialDayForm(manon, '2026-11-25', null), {
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunchOn: true,
		lunchStart: '12:00',
		lunchEnd: '13:00',
		note: '',
	})
	const weekend = initialDayForm(manon, '2026-11-21', null)
	assert.equal(weekend.kind, 'off')
	assert.equal(weekend.start, '08:00')
	assert.equal(weekend.lunchOn, true)
	const off = resolveDay(state, manon, '2026-11-13').override
	assert.equal(initialDayForm(manon, '2026-11-13', off).kind, 'off')
	assert.equal(initialDayForm(manon, '2026-11-13', off).note, 'Pas de mission')
})

test('initialDayForm lit les horaires et la pause du jour par défaut du contrat variable', () => {
	const base = member('manon')
	const current = base.contracts[0]
	const custom = {
		...base,
		contracts: [
			{
				...current,
				schedule: {
					kind: 'variable' as const,
					defaultDay: {
						start: '07:30',
						end: '16:00',
						lunch: { start: '12:00', end: '12:45' },
					},
				},
			},
		],
	}
	const form = initialDayForm(custom, '2026-11-25', null)
	assert.equal(form.start, '07:30')
	assert.equal(form.end, '16:00')
	assert.equal(form.lunchEnd, '12:45')
	const weekend = initialDayForm(custom, '2026-11-21', null)
	assert.equal(weekend.start, '07:30')
	assert.equal(weekend.lunchEnd, '12:45')
})

test('plannedDay n’invente pas d’heures pour un jour à planifier absent', () => {
	const manon = member('manon')
	const sick = state.absences.find((other) => other.id === 'abs-sophie-maladie')
	if (!sick) throw new Error('seed')
	const next = {
		...state,
		absences: [
			...state.absences,
			{
				...sick,
				id: 'abs-test-manon',
				memberId: manon.id,
				startDate: '2026-11-25',
				startPart: 'afternoon' as const,
				endDate: '2026-11-25',
			},
		],
	}
	const day = resolveDay(next, manon, '2026-11-25')
	const lines = dayLines(day, types, manon)
	assert.equal(lines[1].hours, null)
	assert.equal(lines[1].text, 'Arrêt maladie')
	assert.equal(lines[0].text, 'À planifier')
})
