import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	balanceImpact,
	canCancel,
	countModeOption,
	describeDateRange,
	describeSpan,
	filterHistory,
	formatDayAmount,
	formatDayCount,
	HISTORY_ALL,
	isAbsenceStatus,
	partsNote,
	resolveStartDate,
	sortPending,
	TYPE_HUES,
	validateTypeName,
} from '#/pages/hr/lib/rh/absences-view'
import type { AbsenceDraft } from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE
const TODAY = '2026-11-04'

function span(
	startDate: string,
	endDate: string,
	startPart: AbsenceDraft['startPart'] = 'morning',
	endPart: AbsenceDraft['endPart'] = 'afternoon',
) {
	return { startDate, endDate, startPart, endPart }
}

function draft(patch: Partial<AbsenceDraft> = {}): AbsenceDraft {
	return {
		memberId: 'thomas',
		typeId: 'cp',
		...span('2026-12-07', '2026-12-09'),
		comment: '',
		proofName: null,
		...patch,
	}
}

test('le décompte de jours s’accorde en français', () => {
	assert.equal(formatDayCount(0.5), '0,5 jour')
	assert.equal(formatDayCount(1), '1 jour')
	assert.equal(formatDayCount(1.5), '1,5 jour')
	assert.equal(formatDayCount(2), '2 jours')
	assert.equal(formatDayCount(8.5), '8,5 jours')
	assert.equal(formatDayAmount(8.5), '8,5 j')
	assert.equal(formatDayAmount(-1.5), '−1,5 j')
	assert.equal(formatDayAmount(6), '6 j')
})

test('les modes de décompte sont décrits', () => {
	assert.equal(countModeOption('working').label, 'Jours ouvrés')
	assert.equal(countModeOption('workable').short, 'en jours ouvrables')
	assert.equal(countModeOption('calendar').short, 'en jours calendaires')
})

test('une période multi-jours se lit « jeudi 5 au vendredi 6 novembre »', () => {
	assert.equal(
		describeSpan(span('2026-11-05', '2026-11-06')),
		'jeudi 5 au vendredi 6 novembre',
	)
	assert.equal(
		describeSpan(span('2026-11-30', '2026-12-01')),
		'lundi 30 novembre au mardi 1er décembre',
	)
	assert.equal(
		describeSpan(span('2026-12-31', '2027-01-04')),
		'jeudi 31 décembre 2026 au lundi 4 janvier 2027',
	)
})

test('une période d’un jour se lit sans « au »', () => {
	assert.equal(
		describeSpan(span('2026-11-04', '2026-11-04')),
		'mercredi 4 novembre',
	)
})

test('les plages de dates se lisent « du 9 au 13 novembre »', () => {
	assert.equal(
		describeDateRange('2026-11-09', '2026-11-13'),
		'du 9 au 13 novembre',
	)
	assert.equal(describeDateRange('2026-11-09', '2026-11-09'), 'le 9 novembre')
	assert.equal(
		describeDateRange('2026-11-30', '2026-12-02'),
		'du 30 novembre au 2 décembre',
	)
	assert.equal(describeDateRange('2026-12-01', '2026-12-01'), 'le 1er décembre')
})

test('les demi-journées sont signalées', () => {
	assert.equal(partsNote(span('2026-11-04', '2026-11-04')), null)
	assert.equal(partsNote(span('2026-11-05', '2026-11-06')), null)
	assert.equal(
		partsNote(span('2026-11-04', '2026-11-04', 'afternoon', 'afternoon')),
		'Après-midi uniquement',
	)
	assert.equal(
		partsNote(span('2026-11-04', '2026-11-04', 'morning', 'morning')),
		'Matin uniquement',
	)
	assert.equal(
		partsNote(span('2026-11-05', '2026-11-06', 'afternoon', 'afternoon')),
		'Début l’après-midi',
	)
	assert.equal(
		partsNote(span('2026-11-05', '2026-11-06', 'morning', 'morning')),
		'Fin à midi',
	)
	assert.equal(
		partsNote(span('2026-11-05', '2026-11-06', 'afternoon', 'morning')),
		'Début l’après-midi, fin à midi',
	)
})

test('impact sur le solde d’une demande à poser', () => {
	const impact = balanceImpact(state, draft(), TODAY, false)
	assert.ok(impact)
	assert.equal(impact.account.id, 'cp')
	assert.equal(impact.days, 3)
	assert.equal(impact.before - impact.after, 3)
	assert.equal(impact.shortfall, Math.max(0, -impact.after))
})

test('impact d’une demande déjà en attente : le solde avant la remet à plat', () => {
	const julien = state.absences.find(
		(absence) => absence.id === 'abs-julien-novembre',
	)
	assert.ok(julien)
	const impact = balanceImpact(state, julien, TODAY, true)
	assert.ok(impact)
	assert.equal(impact.days, 2)
	assert.equal(impact.before - impact.after, 2)
	const fresh = balanceImpact(
		{
			...state,
			absences: state.absences.filter((other) => other.id !== julien.id),
		},
		julien,
		TODAY,
		false,
	)
	assert.ok(fresh)
	assert.equal(fresh.before, impact.before)
	assert.equal(fresh.after, impact.after)
})

test('pas d’impact quand le motif ne débite aucun compte', () => {
	assert.equal(
		balanceImpact(state, draft({ typeId: 'maladie' }), TODAY, false),
		null,
	)
	assert.equal(
		balanceImpact(state, draft({ memberId: 'inconnu' }), TODAY, false),
		null,
	)
})

test('un solde insuffisant donne un manque positif', () => {
	const impact = balanceImpact(
		state,
		draft({
			...span('2026-12-01', '2026-12-23'),
			memberId: 'sophie',
		}),
		TODAY,
		false,
	)
	assert.ok(impact)
	assert.ok(impact.after < 0)
	assert.equal(impact.shortfall, -impact.after)
})

test('seuls les jours de la période du compte sont débités', () => {
	const impact = balanceImpact(
		state,
		draft({ ...span('2027-05-27', '2027-06-04'), memberId: 'karim' }),
		TODAY,
		false,
	)
	assert.ok(impact)
	assert.equal(impact.days, 3)
})

test('annuler : demandes actives dont la fin n’est pas passée', () => {
	const find = (id: string) => {
		const found = state.absences.find((absence) => absence.id === id)
		if (!found) throw new Error(id)
		return found
	}
	assert.equal(canCancel(find('abs-julien-novembre'), TODAY), true)
	assert.equal(canCancel(find('abs-karim-novembre'), TODAY), true)
	assert.equal(canCancel(find('abs-julien-aout'), TODAY), false)
	assert.equal(canCancel(find('abs-thomas-refus'), TODAY), false)
	assert.equal(canCancel(find('abs-lucie-aprem'), TODAY), true)
})

const everyone = {
	query: '',
	status: HISTORY_ALL,
	memberId: HISTORY_ALL,
	typeId: HISTORY_ALL,
} as const

test('l’historique est trié par date de début décroissante', () => {
	const rows = filterHistory(state, everyone)
	assert.equal(rows.length, state.absences.length)
	for (let index = 1; index < rows.length; index++) {
		assert.ok(rows[index - 1].startDate >= rows[index].startDate)
	}
	assert.equal(rows[0].id, 'abs-baptiste-noel')
})

test('filtres par statut, membre et motif', () => {
	const pending = filterHistory(state, { ...everyone, status: 'pending' })
	assert.ok(pending.length > 0)
	assert.ok(pending.every((absence) => absence.status === 'pending'))
	const thomas = filterHistory(state, { ...everyone, memberId: 'thomas' })
	assert.ok(thomas.every((absence) => absence.memberId === 'thomas'))
	assert.ok(thomas.length >= 3)
	const rtt = filterHistory(state, { ...everyone, typeId: 'rtt' })
	assert.ok(rtt.every((absence) => absence.typeId === 'rtt'))
	const combined = filterHistory(state, {
		...everyone,
		memberId: 'thomas',
		typeId: 'rtt',
		status: 'approved',
	})
	assert.deepEqual(
		combined.map((absence) => absence.id),
		['abs-thomas-rtt'],
	)
})

test('la recherche par nom ignore la casse et les accents', () => {
	const byFirstName = filterHistory(state, { ...everyone, query: 'KARIM' })
	assert.ok(byFirstName.length > 0)
	assert.ok(byFirstName.every((absence) => absence.memberId === 'karim'))
	const byLastName = filterHistory(state, { ...everyone, query: ' roche ' })
	assert.ok(byLastName.every((absence) => absence.memberId === 'julien'))
	assert.equal(filterHistory(state, { ...everyone, query: 'zzz' }).length, 0)
})

test('le nom d’un motif est obligatoire et unique', () => {
	assert.equal(validateTypeName(state, '   '), 'Le nom est obligatoire.')
	assert.equal(
		validateTypeName(state, ' congés PAYÉS '),
		'Un motif porte déjà ce nom.',
	)
	assert.equal(validateTypeName(state, 'Congés payés', 'cp'), null)
	assert.equal(validateTypeName(state, 'Déménagement'), null)
})

test('la palette de couleurs compte une dizaine de teintes distinctes', () => {
	assert.equal(
		new Set(TYPE_HUES.map((entry) => entry.hue)).size,
		TYPE_HUES.length,
	)
	assert.ok(TYPE_HUES.length >= 8 && TYPE_HUES.length <= 12)
	for (const type of state.absenceTypes) {
		assert.ok(
			TYPE_HUES.some((entry) => entry.hue === type.hue),
			`${type.name} : teinte ${type.hue} absente de la palette`,
		)
	}
})

test('une date initiale invalide retombe sur la date du jour', () => {
	assert.equal(resolveStartDate('2026-11-12', TODAY), '2026-11-12')
	assert.equal(resolveStartDate(undefined, TODAY), TODAY)
	assert.equal(resolveStartDate('', TODAY), TODAY)
	assert.equal(resolveStartDate('pas-une-date', TODAY), TODAY)
	assert.equal(resolveStartDate('2026-13-45', TODAY), TODAY)
	assert.equal(resolveStartDate('2026-02-30', TODAY), TODAY)
})

test('les demandes à valider sont triées par début le plus proche', () => {
	const sorted = sortPending(
		state.absences.filter((absence) => absence.status === 'pending'),
	)
	assert.deepEqual(
		sorted.map((absence) => absence.id),
		['abs-julien-novembre', 'abs-thomas-rtt-pending'],
	)
	const extra = {
		...sorted[0],
		id: 'plus-tot',
		startDate: '2026-11-04',
		endDate: '2026-11-04',
	}
	assert.equal(sortPending([...sorted, extra])[0].id, 'plus-tot')
	assert.equal(sorted.length, 2)
})

test('seuls les quatre statuts connus sont acceptés comme filtre d’URL', () => {
	for (const status of ['pending', 'approved', 'rejected', 'cancelled'])
		assert.equal(isAbsenceStatus(status), true)
	assert.equal(isAbsenceStatus('all'), false)
	assert.equal(isAbsenceStatus(undefined), false)
	assert.equal(isAbsenceStatus(3), false)
})
