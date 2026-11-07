import { describe, expect, it } from 'vitest'
import {
	currentMemberOf,
	findPerson,
	personOf,
} from '#/pages/customers/lib/people'

const members = [
	{ id: 'm1', display_name: 'Roche Julien', account: { email: 'j@x.fr' } },
	{ id: 'm2', display_name: 'Martin Lucie', account: null },
	{ id: 'm3', display_name: 'Benali Karim' },
]

describe('personOf', () => {
	it('names a member and gives it a stable hue', () => {
		expect(personOf(members[0], () => 120)).toEqual({
			id: 'm1',
			name: 'Roche Julien',
			hue: 120,
		})
	})
})

describe('findPerson', () => {
	it('finds a person by id, nobody for null or an unknown id', () => {
		const people = members.map((member) => personOf(member, () => 0))
		expect(findPerson(people, 'm2')?.name).toBe('Martin Lucie')
		expect(findPerson(people, null)).toBeUndefined()
		expect(findPerson(people, 'gone')).toBeUndefined()
	})
})

describe('currentMemberOf', () => {
	it('matches the signed-in e-mail ignoring case and spaces', () => {
		expect(currentMemberOf(members, ' J@X.fr ')?.id).toBe('m1')
	})

	it('finds nobody without an e-mail or a matching account', () => {
		expect(currentMemberOf(members, null)).toBeUndefined()
		expect(currentMemberOf(members, '')).toBeUndefined()
		expect(currentMemberOf(members, 'other@x.fr')).toBeUndefined()
	})
})
