import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	emptyProfileFields,
	overlayProfiles,
	type ProfileRecord,
	profileFieldsOf,
	profileRequestOf,
} from '#/pages/hr/lib/rh/member-profile'
import { EMPTY_ADDRESS } from '#/pages/hr/lib/rh/members'

const state = INITIAL_RH_STATE

function record(patch: Partial<ProfileRecord> = {}): ProfileRecord {
	return {
		member_id: 'julien',
		organization_id: 'org-1',
		work_email: 'julien@atelier.fr',
		personal_email: null,
		mobile: '06 12 34 56 78',
		landline: null,
		address: {
			street: '1 rue X',
			complement: '',
			postal_code: '75011',
			city: 'Paris',
			country: 'France',
		},
		job_title: 'Chef de chantier',
		manager_member_id: 'karim',
		work_permit_end_date: '2027-01-31',
		archived_at: null,
		...patch,
	}
}

test('un profil nul devient des champs vides, le pays reste la France', () => {
	const fields = profileFieldsOf(
		record({
			work_email: null,
			mobile: null,
			job_title: null,
			manager_member_id: null,
			work_permit_end_date: null,
			address: {
				street: '',
				complement: '',
				postal_code: null,
				city: '',
				country: 'France',
			},
		}),
	)
	assert.equal(fields.workEmail, '')
	assert.equal(fields.mobile, '')
	assert.equal(fields.jobTitle, '')
	assert.equal(fields.managerId, null)
	assert.equal(fields.workPermitEndDate, null)
	assert.deepEqual(fields.address, EMPTY_ADDRESS)
})

test('les champs vides repartent en null vers l’API', () => {
	const body = profileRequestOf({
		...emptyProfileFields(''),
		workEmail: ' a@b.fr ',
		address: { ...EMPTY_ADDRESS, street: ' 1 rue X ' },
	})
	assert.equal(body.work_email, 'a@b.fr')
	assert.equal(body.personal_email, null)
	assert.equal(body.mobile, null)
	assert.equal(body.landline, null)
	assert.equal(body.job_title, null)
	assert.equal(body.manager_member_id, null)
	assert.equal(body.work_permit_end_date, null)
	assert.equal(body.address?.street, '1 rue X')
	assert.equal(body.address?.postal_code, null)
	assert.equal(body.address?.country, 'France')
})

test('l’overlay remplace les champs de profil et laisse le reste du membre', () => {
	const before = state.members.find((member) => member.id === 'julien')
	assert.ok(before)
	const next = overlayProfiles(state, [record()], new Map())
	const julien = next.members.find((member) => member.id === 'julien')
	assert.ok(julien)
	assert.equal(julien.jobTitle, 'Chef de chantier')
	assert.equal(julien.address.postalCode, '75011')
	assert.equal(julien.workPermitEndDate, '2027-01-31')
	assert.equal(julien.managerId, 'karim')
	assert.equal(julien.status, 'active')
	assert.equal(julien.contracts, before.contracts)
	assert.equal(julien.documents, before.documents)
	assert.equal(julien.hue, before.hue)
})

test('un membre sans profil reçoit les valeurs par défaut, jamais celles du prototype', () => {
	const next = overlayProfiles(state, [], new Map([['julien', 'j@compte.fr']]))
	for (const member of next.members) {
		assert.equal(member.jobTitle, '')
		assert.equal(member.mobile, '')
		assert.equal(member.managerId, null)
		assert.equal(member.workPermitEndDate, null)
		assert.equal(member.status, 'active')
		assert.equal(member.archivedAt, null)
	}
	assert.equal(
		next.members.find((member) => member.id === 'julien')?.workEmail,
		'j@compte.fr',
	)
	assert.equal(
		next.members.find((member) => member.id === 'karim')?.workEmail,
		'',
	)
})

test('archived_at archive le membre à la date du jour de l’archivage', () => {
	const next = overlayProfiles(
		state,
		[record({ archived_at: '2026-11-04T09:30:00Z' })],
		new Map(),
	)
	const julien = next.members.find((member) => member.id === 'julien')
	assert.equal(julien?.status, 'archived')
	assert.equal(julien?.archivedAt, '2026-11-04')
})

test('profileFieldsOf et profileRequestOf font l’aller-retour', () => {
	const source = record()
	const body = profileRequestOf(profileFieldsOf(source))
	assert.equal(body.work_email, source.work_email)
	assert.equal(body.mobile, source.mobile)
	assert.equal(body.job_title, source.job_title)
	assert.equal(body.manager_member_id, source.manager_member_id)
	assert.equal(body.work_permit_end_date, source.work_permit_end_date)
	assert.equal(body.address?.postal_code, '75011')
})
