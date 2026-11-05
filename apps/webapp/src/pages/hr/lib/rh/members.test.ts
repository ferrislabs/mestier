import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	addDocument,
	archiveMember,
	createMember,
	currentSchedule,
	defaultContractSchedule,
	defaultSchedule,
	directReports,
	EMPTY_ADDRESS,
	formatAddress,
	removeDocument,
	restoreMember,
	upcomingDeadlines,
	updateMember,
	validateMemberDraft,
	weeklyHours,
	weeklyHoursOf,
} from '#/pages/hr/lib/rh/members'
import type {
	ActionContext,
	ContractDraft,
	MemberDraft,
} from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE
const context: ActionContext = {
	now: '2026-11-04T10:00:00.000Z',
	today: '2026-11-04',
	actorId: 'baptiste',
	newId: () => 'new-id',
}

function contractDraft(patch: Partial<ContractDraft> = {}): ContractDraft {
	return {
		type: 'cdi',
		startDate: '2026-11-16',
		endDate: null,
		trialEndDate: null,
		schedule: defaultContractSchedule(),
		...patch,
	}
}

function draft(patch: Partial<MemberDraft> = {}): MemberDraft {
	return {
		firstName: ' Nina ',
		lastName: 'Rey',
		workEmail: 'nina.rey@atelier-roche.fr',
		personalEmail: '',
		mobile: '',
		landline: '',
		address: { ...EMPTY_ADDRESS },
		jobTitle: 'Carreleuse',
		contract: contractDraft(),
		workPermitEndDate: null,
		managerId: 'karim',
		...patch,
	}
}

test('weeklyHours déduit la pause déjeuner de chaque journée', () => {
	assert.equal(weeklyHours(defaultSchedule()), 40)
	const hoursOf = (id: string) => {
		const found = state.members.find((m) => m.id === id)
		return found ? weeklyHoursOf(currentSchedule(found, '2026-11-04')) : null
	}
	assert.equal(hoursOf('lucie'), 30)
	assert.equal(hoursOf('yanis'), 26.3)
	assert.equal(hoursOf('hugo'), 36)
	assert.equal(hoursOf('manon'), null)
})

test('validateMemberDraft vérifie identité, e-mail et dates', () => {
	assert.deepEqual(validateMemberDraft(draft()), [])
	assert.equal(validateMemberDraft(draft({ firstName: ' ' })).length, 1)
	assert.equal(validateMemberDraft(draft({ workEmail: 'nina' })).length, 1)
	assert.equal(
		validateMemberDraft(draft({ workEmail: '', personalEmail: '' })).length,
		1,
	)
	assert.deepEqual(
		validateMemberDraft(
			draft({ workEmail: '', personalEmail: 'nina@gmail.com' }),
		),
		[],
	)
	assert.equal(validateMemberDraft(draft({ personalEmail: 'nina' })).length, 1)
	assert.equal(validateMemberDraft(draft({ mobile: '06 12' })).length, 1)
	assert.equal(validateMemberDraft(draft({ landline: 'abc' })).length, 1)
	assert.deepEqual(
		validateMemberDraft(
			draft({ mobile: '+33 6 12 34 56 78', landline: '04 70 31 24 18' }),
		),
		[],
	)
	assert.equal(
		validateMemberDraft(
			draft({ address: { ...EMPTY_ADDRESS, postalCode: '3200' } }),
		).length,
		1,
	)
	assert.equal(
		validateMemberDraft(draft({ contract: contractDraft({ type: 'cdd' }) }))
			.length,
		1,
	)
	assert.equal(
		validateMemberDraft(
			draft({
				contract: contractDraft({ type: 'cdd', endDate: '2026-11-01' }),
			}),
		).length,
		1,
	)
})

test('createMember nettoie les champs et ajoute un membre actif', () => {
	const next = createMember(state, context, draft())
	const created = next.members.at(-1)
	assert.equal(created?.firstName, 'Nina')
	assert.equal(created?.status, 'active')
	assert.equal(created?.documents.length, 0)
	assert.equal(createMember(state, context, draft({ lastName: '' })), state)
})

test('updateMember modifie uniquement le membre ciblé', () => {
	const next = updateMember(state, 'julien', { jobTitle: 'Chef d’équipe pose' })
	assert.equal(
		next.members.find((m) => m.id === 'julien')?.jobTitle,
		'Chef d’équipe pose',
	)
	assert.equal(
		next.members.find((m) => m.id === 'karim')?.jobTitle,
		'Chef de chantier',
	)
})

test('archiveMember reporte les subordonnés sur le responsable et annule les demandes futures', () => {
	const next = archiveMember(state, context, 'julien')
	const julien = next.members.find((m) => m.id === 'julien')
	assert.equal(julien?.status, 'archived')
	assert.equal(julien?.archivedAt, '2026-11-04')
	assert.equal(
		next.members.find((m) => m.id === 'manon')?.managerId,
		'baptiste',
	)
	assert.equal(
		next.absences.find((a) => a.id === 'abs-julien-novembre')?.status,
		'cancelled',
	)
	assert.equal(archiveMember(next, context, 'julien'), next)
})

test('restoreMember réactive un membre archivé', () => {
	const next = restoreMember(state, 'marc')
	assert.equal(next.members.find((m) => m.id === 'marc')?.status, 'active')
	assert.equal(next.members.find((m) => m.id === 'marc')?.archivedAt, null)
})

test('documents : ajout puis suppression', () => {
	const added = addDocument(state, context, 'julien', {
		name: 'Permis CACES.pdf',
		kind: 'diploma',
	})
	const documents =
		added.members.find((m) => m.id === 'julien')?.documents ?? []
	assert.equal(documents.at(-1)?.name, 'Permis CACES.pdf')
	const removed = removeDocument(added, 'julien', 'new-id')
	assert.equal(
		removed.members.find((m) => m.id === 'julien')?.documents.length,
		documents.length - 1,
	)
	assert.equal(
		addDocument(state, context, 'julien', { name: ' ', kind: 'other' }),
		state,
	)
})

test('directReports ne liste que les membres actifs', () => {
	assert.deepEqual(
		directReports(state, 'karim')
			.map((m) => m.id)
			.sort(),
		['hugo', 'sophie', 'thomas', 'yanis'],
	)
})

test('upcomingDeadlines trie les échéances des 60 prochains jours', () => {
	const deadlines = upcomingDeadlines(state, '2026-11-04')
	assert.deepEqual(
		deadlines.map((d) => [d.memberId, d.kind, d.daysLeft]),
		[
			['sophie', 'trial', 2],
			['manon', 'contract_end', 23],
			['hugo', 'work_permit', 41],
			['hugo', 'contract_end', 44],
		],
	)
})

test('formatAddress compose une adresse lisible', () => {
	assert.equal(formatAddress(EMPTY_ADDRESS), '')
	assert.equal(
		formatAddress({
			street: '14 avenue de la Gare',
			complement: 'Bâtiment B',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		}),
		'14 avenue de la Gare, Bâtiment B, 03200 Vichy',
	)
})

test('createMember nettoie e-mails, téléphones et adresse', () => {
	const next = createMember(state, context, {
		...draft(),
		personalEmail: ' nina@gmail.com ',
		mobile: ' 06 12 34 56 78 ',
		address: {
			street: ' 3 rue Neuve ',
			complement: '',
			postalCode: ' 03200 ',
			city: ' Vichy ',
			country: '',
		},
	})
	const created = next.members.at(-1)
	assert.equal(created?.personalEmail, 'nina@gmail.com')
	assert.equal(created?.mobile, '06 12 34 56 78')
	assert.deepEqual(created?.address, {
		street: '3 rue Neuve',
		complement: '',
		postalCode: '03200',
		city: 'Vichy',
		country: 'France',
	})
})
