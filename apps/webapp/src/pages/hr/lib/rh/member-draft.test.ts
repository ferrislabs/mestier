import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { emptyContractDraft } from '#/pages/hr/lib/rh/contract-draft'
import {
	emptyDraft,
	fieldsOf,
	finalizeDraft,
	groupErrors,
	identityOf,
	managerCandidates,
	validateIdentity,
} from '#/pages/hr/lib/rh/member-draft'
import { EMPTY_ADDRESS, validateMemberDraft } from '#/pages/hr/lib/rh/members'

const state = INITIAL_RH_STATE

test('le brouillon vide est invalide et contient un contrat valide', () => {
	const draft = emptyDraft('2026-11-04')
	assert.equal(draft.contract.startDate, '2026-11-04')
	assert.equal(draft.managerId, null)
	assert.equal(draft.workPermitEndDate, null)
	assert.deepEqual(draft.address, EMPTY_ADDRESS)
	assert.notEqual(draft.address, EMPTY_ADDRESS)
	assert.deepEqual(draft.contract, emptyContractDraft('2026-11-04'))
	assert.ok(validateMemberDraft(draft).length > 0)
})

test('fieldsOf reprend l’identité complète d’un membre sans contrat', () => {
	const hugo = state.members.find((member) => member.id === 'hugo')
	assert.ok(hugo)
	const fields = fieldsOf(hugo)
	assert.equal(fields.firstName, 'Hugo')
	assert.equal(fields.managerId, 'karim')
	assert.equal(fields.workPermitEndDate, '2026-12-15')
	assert.equal(fields.workEmail, hugo.workEmail)
	assert.equal(fields.personalEmail, '')
	assert.equal(fields.mobile, hugo.mobile)
	assert.deepEqual(fields.address, hugo.address)
	assert.notEqual(fields.address, hugo.address)
	assert.equal('contract' in fields, false)
	assert.equal('id' in fields, false)
	assert.equal('documents' in fields, false)
})

test('identityOf retire le contrat d’un brouillon', () => {
	const fields = identityOf(emptyDraft('2026-11-04'))
	assert.equal('contract' in fields, false)
	assert.deepEqual(fields.address, EMPTY_ADDRESS)
})

test('finalizeDraft nettoie les textes, l’adresse, le titre de travail et le contrat', () => {
	const draft = {
		...emptyDraft('2026-11-04'),
		firstName: '  Nina ',
		lastName: ' Rey',
		workEmail: ' nina@atelier.fr ',
		personalEmail: ' nina@mail.fr ',
		mobile: ' 06 11 22 33 44 ',
		landline: ' ',
		address: {
			street: ' 4 rue des Lilas ',
			complement: ' ',
			postalCode: ' 69003 ',
			city: ' Lyon ',
			country: ' ',
		},
		jobTitle: ' Carreleuse ',
		workPermitEndDate: '',
		contract: {
			...emptyContractDraft('2026-11-04'),
			type: 'cdd' as const,
			endDate: '',
			trialEndDate: '',
		},
	}
	const result = finalizeDraft(draft)
	assert.equal(result.firstName, 'Nina')
	assert.equal(result.lastName, 'Rey')
	assert.equal(result.workEmail, 'nina@atelier.fr')
	assert.equal(result.personalEmail, 'nina@mail.fr')
	assert.equal(result.mobile, '06 11 22 33 44')
	assert.equal(result.landline, '')
	assert.deepEqual(result.address, {
		street: '4 rue des Lilas',
		complement: '',
		postalCode: '69003',
		city: 'Lyon',
		country: 'France',
	})
	assert.equal(result.jobTitle, 'Carreleuse')
	assert.equal(result.workPermitEndDate, null)
	assert.equal(result.contract.endDate, null)
	assert.equal(result.contract.trialEndDate, null)
})

test('groupErrors associe chaque message d’identité à son champ', () => {
	const grouped = groupErrors(
		validateMemberDraft(finalizeDraft(emptyDraft('2026-11-04'))),
	)
	assert.equal(grouped.byField.firstName, 'Le prénom est obligatoire.')
	assert.equal(grouped.byField.lastName, 'Le nom est obligatoire.')
	assert.equal(grouped.byField.emails, 'Renseigne au moins une adresse e-mail.')
	assert.deepEqual(grouped.other, [])
	assert.deepEqual(groupErrors(['Quelque chose a échoué.']).other, [
		'Quelque chose a échoué.',
	])
})

test('groupErrors couvre les e-mails, téléphones et code postal', () => {
	const draft = {
		...emptyDraft('2026-11-04'),
		firstName: 'Nina',
		lastName: 'Rey',
		workEmail: 'pas-un-email',
		personalEmail: 'autre@',
		mobile: 'abc',
		landline: '12',
		address: { ...EMPTY_ADDRESS, postalCode: '123' },
	}
	const grouped = groupErrors(validateMemberDraft(draft))
	assert.deepEqual(grouped.other, [])
	assert.deepEqual(Object.keys(grouped.byField).sort(), [
		'landline',
		'mobile',
		'personalEmail',
		'postalCode',
		'workEmail',
	])
})

test('groupErrors laisse les erreurs de contrat dans les autres messages', () => {
	const draft = {
		...emptyDraft('2026-11-04'),
		firstName: 'Nina',
		lastName: 'Rey',
		workEmail: 'nina@atelier.fr',
		contract: { ...emptyContractDraft('2026-11-04'), type: 'cdd' as const },
	}
	const grouped = groupErrors(validateMemberDraft(draft))
	assert.deepEqual(grouped.byField, {})
	assert.deepEqual(grouped.other, [
		'Ce type de contrat demande une date de fin.',
	])
})

test('validateIdentity ne contrôle que l’identité et les coordonnées', () => {
	const fields = fieldsOf(
		state.members.find((member) => member.id === 'karim') ?? state.members[0],
	)
	assert.deepEqual(validateIdentity(fields), [])
	assert.deepEqual(validateIdentity({ ...fields, firstName: ' ' }), [
		'Le prénom est obligatoire.',
	])
	assert.deepEqual(validateIdentity({ ...fields, personalEmail: 'nope' }), [
		'L’adresse e-mail personnelle n’est pas valide.',
	])
	assert.deepEqual(
		validateIdentity({ ...fields, workEmail: '', personalEmail: '' }),
		['Renseigne au moins une adresse e-mail.'],
	)
})

test('une seule adresse e-mail suffit', () => {
	const hugo = state.members.find((member) => member.id === 'hugo')
	assert.ok(hugo)
	assert.deepEqual(validateIdentity(fieldsOf(hugo)), [])
	assert.deepEqual(
		validateIdentity({
			...fieldsOf(hugo),
			workEmail: '',
			personalEmail: 'h@x.fr',
		}),
		[],
	)
})

test('managerCandidates exclut le membre, son équipe, les archivés', () => {
	const forKarim = managerCandidates(state, 'karim').map((member) => member.id)
	assert.ok(!forKarim.includes('karim'))
	assert.ok(!forKarim.includes('sophie'))
	assert.ok(!forKarim.includes('thomas'))
	assert.ok(!forKarim.includes('marc'))
	assert.ok(forKarim.includes('baptiste'))
	assert.ok(forKarim.includes('lucie'))
	assert.ok(forKarim.includes('manon'))
})

test('managerCandidates exclut aussi les descendants indirects', () => {
	const forBaptiste = managerCandidates(state, 'baptiste').map(
		(member) => member.id,
	)
	assert.deepEqual(forBaptiste, [])
})

test('managerCandidates liste tous les actifs en création', () => {
	const all = managerCandidates(state, null).map((member) => member.id)
	assert.equal(all.length, 9)
	assert.ok(!all.includes('marc'))
})
