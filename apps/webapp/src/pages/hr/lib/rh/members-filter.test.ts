import { assert, test } from 'vitest'
import {
	DEFAULT_FILTERS,
	filterMembers,
	membersSummary,
} from '#/pages/hr/lib/rh/members-filter'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const members = INITIAL_RH_STATE.members

const TODAY = '2026-11-04'

function ids(filters: Partial<typeof DEFAULT_FILTERS>, today = TODAY) {
	return filterMembers(members, { ...DEFAULT_FILTERS, ...filters }, today).map(
		(member) => member.id,
	)
}

test('le filtre par défaut ne garde que les membres actifs', () => {
	const result = ids({})
	assert.ok(result.length > 0)
	assert.ok(!result.includes('marc'))
	assert.equal(DEFAULT_FILTERS.status, 'active')
})

test('le filtre archivés ne garde que les archivés', () => {
	assert.deepEqual(ids({ status: 'archived' }), ['marc'])
})

test('le filtre tous place les archivés en dernier', () => {
	const result = ids({ status: 'all' })
	assert.equal(result.length, members.length)
	assert.equal(result.at(-1), 'marc')
})

test('la recherche ignore la casse et les accents', () => {
	assert.deepEqual(ids({ query: 'ELECTRICIEN' }), ['thomas'])
	assert.deepEqual(ids({ query: 'plombiere' }), ['sophie'])
})

test('la recherche couvre le nom, le poste et l’e-mail', () => {
	assert.deepEqual(ids({ query: 'benali' }), ['karim'])
	assert.deepEqual(ids({ query: 'chef de chantier' }), ['karim'])
	assert.deepEqual(ids({ query: 'julien.roche@' }), ['julien'])
})

test('la recherche couvre l’e-mail personnel et le mobile', () => {
	assert.deepEqual(ids({ query: 'gmail.com' }).includes('karim'), true)
	assert.deepEqual(ids({ query: 'proton.me' }), ['sophie'])
	assert.deepEqual(ids({ query: '06 41 22' }), ['baptiste'])
})

test('la recherche d’un numéro ignore les espaces et les points', () => {
	assert.deepEqual(ids({ query: '0641221890' }), ['baptiste'])
	assert.deepEqual(ids({ query: '06.72.15.40.03' }), ['karim'])
	assert.deepEqual(ids({ query: '07 83 52' }), ['yanis'])
})

test('la recherche couvre le prénom et le nom ensemble', () => {
	assert.deepEqual(ids({ query: 'sophie lambert' }), ['sophie'])
})

test('le filtre contrat restreint la liste', () => {
	assert.deepEqual(ids({ contract: 'cdd' }), ['hugo'])
	assert.deepEqual(ids({ contract: 'interim' }), ['manon'])
})

test('les filtres se combinent', () => {
	assert.deepEqual(ids({ contract: 'cdi', query: 'karim' }), ['karim'])
	assert.deepEqual(ids({ contract: 'cdd', query: 'karim' }), [])
})

test('le résumé compte les actifs et les archivés', () => {
	assert.equal(membersSummary(members), '9 membres actifs, 1 archivé')
	assert.equal(membersSummary(members.slice(0, 1)), '1 membre actif')
	assert.equal(membersSummary([]), 'Aucun membre actif')
})

test('le filtre contrat porte sur le contrat en cours', () => {
	assert.deepEqual(ids({ contract: 'apprentissage' }), ['yanis'])
	assert.ok(ids({ contract: 'cdi' }).includes('julien'))
	assert.ok(!ids({ contract: 'cdd' }).includes('julien'))
	assert.ok(!ids({ contract: 'apprentissage' }).includes('thomas'))
})

test('le filtre contrat suit la date du jour', () => {
	assert.ok(ids({ contract: 'cdd' }, '2021-10-15').includes('julien'))
	assert.ok(ids({ contract: 'apprentissage' }, '2021-06-15').includes('julien'))
})
