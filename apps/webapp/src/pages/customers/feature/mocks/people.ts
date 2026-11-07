export interface Person {
	id: string
	name: string
	role: string
	hue: number
}

export const CURRENT_PERSON_ID = 'baptiste'

export const PEOPLE: readonly Person[] = [
	{ id: 'baptiste', name: 'Baptiste Parmantier', role: 'Gérant', hue: 250 },
	{ id: 'julien', name: 'Julien Roche', role: 'SAV et pose', hue: 155 },
	{ id: 'lucie', name: 'Lucie Martin', role: 'Administration', hue: 350 },
	{ id: 'karim', name: 'Karim Benali', role: 'Chef de chantier', hue: 35 },
]

export function findPerson(id: string | null): Person | undefined {
	return PEOPLE.find((person) => person.id === id)
}
