import type { Schemas } from '#/api/api.client'

export interface Person {
	id: string
	name: string
	hue: number
}

type Member = Pick<Schemas.MemberResponse, 'id' | 'display_name'> & {
	account?: Pick<NonNullable<Schemas.MemberResponse['account']>, 'email'> | null
}

export function personOf(member: Member, hueOf: (seed: string) => number) {
	return {
		id: member.id,
		name: member.display_name,
		hue: hueOf(member.id),
	} satisfies Person
}

export function findPerson(
	people: readonly Person[],
	id: string | null,
): Person | undefined {
	return people.find((person) => person.id === id)
}

export function currentMemberOf(
	members: readonly Member[],
	email: string | null | undefined,
): Member | undefined {
	const wanted = email?.trim().toLowerCase()
	if (!wanted) return undefined
	return members.find(
		(member) => member.account?.email.trim().toLowerCase() === wanted,
	)
}
