export const CURSUS_ENTRY = {
	promotions: 'promotions',
	triads: 'triads',
	myTriads: 'myTriads',
	apprentices: 'apprentices',
	tutors: 'tutors',
	companies: 'companies',
	trainings: 'trainings',
	establishments: 'establishments',
} as const

export type CursusEntry = (typeof CURSUS_ENTRY)[keyof typeof CURSUS_ENTRY]

const SECTIONS: readonly (readonly [string, CursusEntry])[] = [
	['/cursus/promotions', CURSUS_ENTRY.promotions],
	['/cursus/trinomes', CURSUS_ENTRY.triads],
	['/cursus/mes-trinomes', CURSUS_ENTRY.myTriads],
	['/cursus/apprentis', CURSUS_ENTRY.apprentices],
	['/cursus/tuteurs', CURSUS_ENTRY.tutors],
	['/cursus/entreprises', CURSUS_ENTRY.companies],
	['/cursus/formations', CURSUS_ENTRY.trainings],
	['/cursus/etablissements', CURSUS_ENTRY.establishments],
]

function isUnder(modulePath: string, prefix: string): boolean {
	return modulePath === prefix || modulePath.startsWith(`${prefix}/`)
}

export function activeCursusEntry(modulePath: string): CursusEntry | null {
	const section = SECTIONS.find(([prefix]) => isUnder(modulePath, prefix))
	return section ? section[1] : null
}
