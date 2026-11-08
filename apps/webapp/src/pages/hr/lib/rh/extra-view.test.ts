import { assert, test } from 'vitest'
import { validateExtra } from '#/pages/hr/lib/rh/extra'
import {
	bonusMinutes,
	buildExtraDraft,
	CUSTOM_RATE,
	canCancel,
	canCancelOwn,
	dayCaption,
	describeBonus,
	describeEquivalent,
	describeHourExample,
	faultyField,
	filterHistory,
	HISTORY_ALL,
	holidayNote,
	isRateInvalid,
	kindLabel,
	netMinutes,
	paidMinutes,
	parseRate,
	resolveDate,
	resolveMajoration,
	SURCHARGE_KINDS,
	shiftMonth,
	sortPending,
	suggestedMajoration,
} from '#/pages/hr/lib/rh/extra-view'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE

function extraOf(patch: Partial<ExtraPeriod> = {}): ExtraPeriod {
	return {
		id: 'x',
		memberId: 'thomas',
		date: '2026-11-21',
		start: '08:00',
		end: '12:00',
		lunch: null,
		reason: 'Test',
		surchargeId: 'hs25',
		rate: 25,
		status: 'pending',
		createdAt: '2026-11-04T08:00:00.000Z',
		createdById: 'thomas',
		decidedById: null,
		decidedAt: null,
		decisionComment: '',
		...patch,
	}
}

function withExtras(extras: readonly ExtraPeriod[]): RhState {
	return { ...state, extras }
}

test('les durées et la différence de majoration se calculent', () => {
	const span = { start: '08:00', end: '10:30', lunch: null, rate: 25 }
	assert.equal(netMinutes(span), 150)
	assert.equal(paidMinutes(span), 188)
	assert.equal(bonusMinutes(span), 38)
	assert.equal(
		bonusMinutes({ start: '08:00', end: '12:00', lunch: null, rate: 0 }),
		0,
	)
})

test('une plage incomplète ou incohérente ne donne aucune durée', () => {
	assert.equal(netMinutes({ start: '', end: '12:00', lunch: null, rate: 0 }), 0)
	assert.equal(
		netMinutes({ start: '12:00', end: '08:00', lunch: null, rate: 0 }),
		0,
	)
	assert.equal(
		paidMinutes({ start: '08:00', end: '', lunch: null, rate: Number.NaN }),
		0,
	)
	assert.equal(
		paidMinutes({
			start: '08:00',
			end: '10:00',
			lunch: null,
			rate: Number.NaN,
		}),
		120,
	)
})

test('la phrase d’équivalence compare durée nette et durée payée', () => {
	assert.equal(
		describeEquivalent({ start: '08:00', end: '10:30', lunch: null, rate: 25 }),
		'2 h 30 → 3 h 08 payées',
	)
	assert.equal(
		describeEquivalent({ start: '08:00', end: '12:00', lunch: null, rate: 25 }),
		'4 h → 5 h payées',
	)
})

test('la part de majoration est décrite', () => {
	assert.equal(describeBonus(45), 'dont 45 min de majoration')
	assert.equal(describeBonus(75), 'dont 1 h 15 de majoration')
	assert.equal(describeBonus(0), 'Aucune majoration')
})

test('seuls les jours fériés reçoivent une mention avec leur nom', () => {
	assert.equal(holidayNote('2026-11-11'), 'Jour férié : Armistice 1918')
	assert.equal(holidayNote('2026-11-01'), 'Jour férié : Toussaint')
	assert.equal(holidayNote('2026-11-22'), null)
	assert.equal(holidayNote('2026-11-21'), null)
	assert.equal(holidayNote(''), null)
})

test('le taux saisi est lu avec virgule ou point', () => {
	assert.equal(parseRate('30'), 30)
	assert.equal(parseRate(' 12,5 '), 12.5)
	assert.equal(parseRate('0'), 0)
	assert.ok(Number.isNaN(parseRate('')))
	assert.ok(Number.isNaN(parseRate('abc')))
})

test('la majoration suggérée suit le jour de la date', () => {
	assert.deepEqual(suggestedMajoration(state, '2026-11-21'), {
		enabled: true,
		choice: 'hs25',
		custom: '',
	})
	assert.deepEqual(suggestedMajoration(state, '2026-11-22'), {
		enabled: true,
		choice: 'dimanche',
		custom: '',
	})
	assert.deepEqual(suggestedMajoration(state, '2026-11-11'), {
		enabled: true,
		choice: 'ferie',
		custom: '',
	})
})

test('sans barème disponible la majoration est désactivée', () => {
	const archived: RhState = {
		...state,
		surcharges: state.surcharges.map((surcharge) => ({
			...surcharge,
			archived: true,
		})),
	}
	assert.deepEqual(suggestedMajoration(archived, '2026-11-21'), {
		enabled: false,
		choice: CUSTOM_RATE,
		custom: '',
	})
})

test('le choix de majoration se traduit en taux et barème', () => {
	assert.deepEqual(
		resolveMajoration(state, { enabled: true, choice: 'dimanche', custom: '' }),
		{ surchargeId: 'dimanche', rate: 100 },
	)
	assert.deepEqual(
		resolveMajoration(state, {
			enabled: false,
			choice: 'dimanche',
			custom: '',
		}),
		{ surchargeId: null, rate: 0 },
	)
	assert.deepEqual(
		resolveMajoration(state, {
			enabled: true,
			choice: CUSTOM_RATE,
			custom: '30',
		}),
		{ surchargeId: null, rate: 30 },
	)
	assert.ok(
		Number.isNaN(
			resolveMajoration(state, {
				enabled: true,
				choice: CUSTOM_RATE,
				custom: '',
			}).rate,
		),
	)
	assert.deepEqual(
		resolveMajoration(state, { enabled: true, choice: 'inconnu', custom: '' }),
		{ surchargeId: null, rate: 0 },
	)
})

test('le brouillon construit est accepté par la validation du socle', () => {
	const built = buildExtraDraft(state, {
		memberId: 'thomas',
		date: '2026-11-21',
		start: '08:00',
		end: '12:00',
		lunchOn: false,
		lunch: { start: '12:00', end: '13:00' },
		reason: 'Rattrapage',
		majoration: { enabled: true, choice: 'hs25', custom: '' },
	})
	assert.equal(built.lunch, null)
	assert.equal(built.surchargeId, 'hs25')
	assert.equal(built.rate, 25)
	assert.deepEqual(validateExtra(state, built), [])
})

test('la pause n’est conservée que si elle est activée', () => {
	const form = {
		memberId: 'thomas',
		date: '2026-11-21',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
		reason: 'Rattrapage',
		majoration: { enabled: false, choice: 'hs25', custom: '' },
	}
	assert.deepEqual(buildExtraDraft(state, { ...form, lunchOn: true }).lunch, {
		start: '12:00',
		end: '13:00',
	})
	assert.equal(buildExtraDraft(state, { ...form, lunchOn: false }).lunch, null)
})

test('une date invalide revient à la valeur de repli', () => {
	assert.equal(resolveDate('2026-11-21', '2026-11-04'), '2026-11-21')
	assert.equal(resolveDate('2026-13-45', '2026-11-04'), '2026-11-04')
	assert.equal(resolveDate(undefined, '2026-11-04'), '2026-11-04')
	assert.equal(resolveDate('demain', '2026-11-04'), '2026-11-04')
})

test('le changement de mois reste calé sur le premier jour', () => {
	assert.equal(shiftMonth('2026-11-01', 1), '2026-12-01')
	assert.equal(shiftMonth('2026-11-01', -1), '2026-10-01')
	assert.equal(shiftMonth('2026-12-01', 1), '2027-01-01')
	assert.equal(shiftMonth('2026-11-17', 0), '2026-11-01')
})

test('annuler suppose une déclaration active dont la date n’est pas passée', () => {
	assert.equal(canCancel(extraOf({ date: '2026-11-04' }), '2026-11-04'), true)
	assert.equal(canCancel(extraOf({ date: '2026-11-03' }), '2026-11-04'), false)
	assert.equal(
		canCancel(
			extraOf({ date: '2026-11-21', status: 'approved' }),
			'2026-11-04',
		),
		true,
	)
	assert.equal(
		canCancel(
			extraOf({ date: '2026-11-21', status: 'rejected' }),
			'2026-11-04',
		),
		false,
	)
	assert.equal(
		canCancel(
			extraOf({ date: '2026-11-21', status: 'cancelled' }),
			'2026-11-04',
		),
		false,
	)
})

test('les demandes à valider sont triées par date puis par heure', () => {
	const sorted = sortPending([
		extraOf({ id: 'c', date: '2026-11-21', start: '14:00' }),
		extraOf({ id: 'a', date: '2026-11-11', start: '08:00' }),
		extraOf({ id: 'b', date: '2026-11-21', start: '08:00' }),
		extraOf({
			id: 'd',
			date: '2026-11-21',
			start: '08:00',
			createdAt: '2026-11-03T08:00:00.000Z',
		}),
	])
	assert.deepEqual(
		sorted.map((entry) => entry.id),
		['a', 'd', 'b', 'c'],
	)
})

test('l’historique se filtre par statut, membre et nom puis se trie par date décroissante', () => {
	const all = filterHistory(state, {
		query: '',
		status: HISTORY_ALL,
		memberId: HISTORY_ALL,
	})
	assert.equal(all.length, state.extras.length)
	const dates = all.map((entry) => entry.date)
	assert.deepEqual(dates, [...dates].sort().reverse())
	assert.deepEqual(
		filterHistory(state, {
			query: '',
			status: 'pending',
			memberId: HISTORY_ALL,
		}).map((entry) => entry.id),
		['extra-hugo-samedi', 'extra-sophie-ferie'],
	)
	assert.deepEqual(
		filterHistory(state, {
			query: '',
			status: HISTORY_ALL,
			memberId: 'thomas',
		}).map((entry) => entry.id),
		['extra-thomas-samedi', 'extra-thomas-depannage'],
	)
	assert.deepEqual(
		filterHistory(state, {
			query: '  JULIEN ',
			status: HISTORY_ALL,
			memberId: HISTORY_ALL,
		}).map((entry) => entry.id),
		['extra-julien-soir'],
	)
	assert.deepEqual(
		filterHistory(state, {
			query: 'zzz',
			status: HISTORY_ALL,
			memberId: HISTORY_ALL,
		}),
		[],
	)
})

test('la recherche ignore les accents et la casse', () => {
	const custom = withExtras([extraOf({ memberId: 'lucie' })])
	for (const query of ['lucie', 'LUCIÉ', ' Martin ']) {
		assert.equal(
			filterHistory(custom, {
				query,
				status: HISTORY_ALL,
				memberId: HISTORY_ALL,
			}).length,
			1,
		)
	}
})

test('les types de majoration ont un libellé français', () => {
	assert.deepEqual(
		SURCHARGE_KINDS.map((entry) => entry.label),
		['Heures supplémentaires', 'Dimanche', 'Jour férié', 'Nuit', 'Autre'],
	)
	assert.equal(kindLabel('holiday'), 'Jour férié')
	assert.equal(kindLabel('night'), 'Nuit')
})

test('la légende du jour donne le férié ou le jour de la semaine', () => {
	assert.equal(dayCaption('2026-11-11'), 'Armistice 1918')
	assert.equal(dayCaption('2026-11-22'), 'Dimanche')
	assert.equal(dayCaption('2026-11-21'), 'Samedi')
	assert.equal(dayCaption(''), '')
})

test('l’exemple d’une heure montre l’effet du taux', () => {
	assert.equal(
		describeHourExample(25),
		'Par exemple, 1 h travaillée sera payée 1 h 15.',
	)
	assert.equal(
		describeHourExample(100),
		'Par exemple, 1 h travaillée sera payée 2 h.',
	)
	assert.equal(
		describeHourExample(Number.NaN),
		'Par exemple, 1 h travaillée sera payée 1 h.',
	)
})

test('un taux est valide entre 0 et le maximum', () => {
	assert.equal(isRateInvalid(0), false)
	assert.equal(isRateInvalid(25), false)
	assert.equal(isRateInvalid(200), false)
	assert.equal(isRateInvalid(200.5), true)
	assert.equal(isRateInvalid(-1), true)
	assert.equal(isRateInvalid(Number.NaN), true)
})

test('le champ fautif d’une majoration est désigné', () => {
	assert.equal(faultyField('', 25, true), 'name')
	assert.equal(faultyField('', Number.NaN, true), 'name')
	assert.equal(faultyField('Nuit', Number.NaN, true), 'rate')
	assert.equal(faultyField('Nuit', 0, true), 'rate')
	assert.equal(faultyField('Nuit', 201, true), 'rate')
	assert.equal(faultyField('Nuit', 25, true), 'name')
	assert.equal(faultyField('Nuit', 25, false), null)
})

test('spec_rh_7_6 le créateur annule sa propre déclaration en attente, pas celle d’un autre ni une déclaration tranchée', () => {
	const own = extraOf({ createdById: 'me', date: '2026-12-01' })
	assert.equal(canCancelOwn(own, 'me', '2026-11-04'), true)
	assert.equal(
		canCancelOwn({ ...own, createdById: 'other' }, 'me', '2026-11-04'),
		false,
	)
	assert.equal(
		canCancelOwn({ ...own, status: 'approved' }, 'me', '2026-11-04'),
		false,
	)
	assert.equal(
		canCancelOwn({ ...own, date: '2026-10-01' }, 'me', '2026-11-04'),
		false,
	)
	assert.equal(canCancelOwn(own, '', '2026-11-04'), false)
})
