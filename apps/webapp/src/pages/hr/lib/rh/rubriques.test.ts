import { assert, test } from 'vitest'
import {
	absenceLists,
	contractsSummary,
	documentsSummary,
	identitySummary,
	isMemberTab,
	MEMBER_TABS,
	scheduleSummary,
} from '#/pages/hr/lib/rh/rubriques'
import type { Member } from '#/pages/hr/lib/rh/types'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const TODAY = '2026-11-04'

function memberOf(id: string): Member {
	const found = state.members.find((member) => member.id === id)
	if (!found) throw new Error(id)
	return found
}

test('les quatre rubriques sont déclarées dans l’ordre', () => {
	assert.deepEqual(
		MEMBER_TABS.map((tab) => tab.id),
		['identite', 'contrats', 'documents', 'emploi-du-temps'],
	)
	assert.equal(isMemberTab('contrats'), true)
	assert.equal(isMemberTab('absences'), false)
	assert.equal(isMemberTab(undefined), false)
})

test('identitySummary donne le mobile et la ville', () => {
	const baptiste = memberOf('baptiste')
	assert.equal(
		identitySummary(baptiste),
		[baptiste.mobile, baptiste.address.city].filter(Boolean).join(' · '),
	)
	assert.ok(identitySummary(baptiste).includes(baptiste.mobile))
})

test('identitySummary se rabat sur l’e-mail puis sur un message', () => {
	const karim = memberOf('karim')
	const noPhone = {
		...karim,
		mobile: '',
		address: { ...karim.address, city: '' },
	}
	assert.equal(identitySummary(noPhone), karim.workEmail || karim.personalEmail)
	assert.equal(
		identitySummary({ ...noPhone, workEmail: '', personalEmail: '' }),
		'Coordonnées à compléter',
	)
	assert.equal(
		identitySummary({
			...karim,
			mobile: '',
			address: { ...karim.address, city: 'Lyon' },
		}),
		'Lyon',
	)
})

test('contractsSummary donne le parcours et le nombre', () => {
	assert.equal(
		contractsSummary(memberOf('julien')),
		'Apprentissage → CDD → CDI · 3 contrats',
	)
	assert.equal(contractsSummary(memberOf('karim')), 'CDI · 1 contrat')
	assert.equal(contractsSummary(memberOf('lucie')), 'CDI · 2 contrats')
})

test('documentsSummary pluralise', () => {
	assert.equal(documentsSummary(0), 'Aucun document')
	assert.equal(documentsSummary(1), '1 document')
	assert.equal(documentsSummary(3), '3 documents')
})

test('scheduleSummary donne les heures nettes et les absences à venir', () => {
	assert.equal(
		scheduleSummary(state, memberOf('karim'), TODAY),
		'40 h par semaine · 1 absence à venir',
	)
	assert.equal(
		scheduleSummary(state, memberOf('lucie'), TODAY),
		'30 h par semaine · 1 absence à venir',
	)
	assert.equal(
		scheduleSummary(state, memberOf('hugo'), TODAY),
		'≈ 36 h par semaine en moyenne · aucune absence à venir',
	)
})

test('scheduleSummary décrit un planning variable', () => {
	assert.equal(
		scheduleSummary(state, memberOf('manon'), TODAY),
		'Planning variable · aucune absence à venir',
	)
})

test('absenceLists sépare les absences à venir et passées', () => {
	const lists = absenceLists(state, 'karim', TODAY)
	assert.deepEqual(
		lists.upcoming.map((absence) => absence.startDate),
		['2026-11-09'],
	)
	assert.deepEqual(
		lists.past.map((absence) => absence.startDate),
		['2026-09-18', '2026-08-17'],
	)
})

test('absenceLists compte une absence en cours comme à venir', () => {
	const lists = absenceLists(state, 'sophie', TODAY)
	assert.deepEqual(lists.upcoming.map((absence) => absence.id).length, 1)
	assert.equal(lists.past.length, 0)
})

test('absenceLists écarte les absences refusées ou annulées', () => {
	const lists = absenceLists(state, 'thomas', TODAY)
	assert.ok(
		[...lists.upcoming, ...lists.past].every(
			(absence) => absence.status !== 'rejected',
		),
	)
	assert.deepEqual(
		lists.upcoming.map((absence) => absence.startDate),
		['2026-11-20', '2026-11-27'],
	)
})

test('absenceLists limite chaque liste à 5 éléments', () => {
	const base = state.absences.find((absence) => absence.memberId === 'karim')
	assert.ok(base)
	const many = Array.from({ length: 9 }, (_, index) => ({
		...base,
		id: `many-${index}`,
		startDate: `2026-12-${String(10 + index).padStart(2, '0')}`,
		endDate: `2026-12-${String(10 + index).padStart(2, '0')}`,
	}))
	const lists = absenceLists({ ...state, absences: many }, 'karim', TODAY)
	assert.equal(lists.upcoming.length, 5)
	assert.equal(lists.upcoming[0].startDate, '2026-12-10')
})
