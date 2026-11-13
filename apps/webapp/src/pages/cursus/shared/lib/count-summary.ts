import type { ArchiveFilter } from '#/pages/cursus/shared/lib/list-filters'

export type Gender = 'masculine' | 'feminine'

export interface CountNoun {
	gender: Gender
	one: string
	many: string
}

export type CountQualifier =
	| 'active'
	| 'archived'
	| 'open'
	| 'closed'
	| 'pending'
	| 'inactive'

const QUALIFIERS: Record<
	CountQualifier,
	Record<Gender, { one: string; many: string }>
> = {
	active: {
		masculine: { one: 'actif', many: 'actifs' },
		feminine: { one: 'active', many: 'actives' },
	},
	archived: {
		masculine: { one: 'archivé', many: 'archivés' },
		feminine: { one: 'archivée', many: 'archivées' },
	},
	open: {
		masculine: { one: 'ouvert', many: 'ouverts' },
		feminine: { one: 'ouverte', many: 'ouvertes' },
	},
	closed: {
		masculine: { one: 'clos', many: 'clos' },
		feminine: { one: 'close', many: 'closes' },
	},
	pending: {
		masculine: { one: 'en attente', many: 'en attente' },
		feminine: { one: 'en attente', many: 'en attente' },
	},
	inactive: {
		masculine: { one: 'inactif', many: 'inactifs' },
		feminine: { one: 'inactive', many: 'inactives' },
	},
}

export const PROMOTION_NOUN: CountNoun = {
	gender: 'feminine',
	one: 'promotion',
	many: 'promotions',
}
export const TRIAD_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'trinôme',
	many: 'trinômes',
}
export const PARTICIPANT_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'participant',
	many: 'participants',
}
export const APPRENTICE_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'apprenti',
	many: 'apprentis',
}
export const COMPANY_TUTOR_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'tuteur entreprise',
	many: 'tuteurs entreprise',
}
export const SCHOOL_TUTOR_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'tuteur école',
	many: 'tuteurs école',
}
export const HOST_COMPANY_NOUN: CountNoun = {
	gender: 'feminine',
	one: 'entreprise',
	many: 'entreprises',
}
export const TRAINING_NOUN: CountNoun = {
	gender: 'feminine',
	one: 'formation',
	many: 'formations',
}
export const ESTABLISHMENT_NOUN: CountNoun = {
	gender: 'masculine',
	one: 'établissement',
	many: 'établissements',
}

export function countSummary(
	count: number,
	noun: CountNoun,
	qualifier?: CountQualifier,
): string {
	const plural = count > 1
	const words = [plural ? noun.many : noun.one]
	if (qualifier) {
		const forms = QUALIFIERS[qualifier][noun.gender]
		words.push(plural ? forms.many : forms.one)
	}
	if (count === 0) {
		const none = noun.gender === 'feminine' ? 'Aucune' : 'Aucun'
		return [none, ...words].join(' ')
	}
	return [String(count), ...words].join(' ')
}

export function archiveQualifier(
	filter: ArchiveFilter,
): CountQualifier | undefined {
	return filter === 'all' ? undefined : filter
}
