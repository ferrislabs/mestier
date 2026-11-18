import {
	AlignLeft,
	AtSign,
	Calendar,
	CalendarClock,
	Camera,
	CircleDot,
	Hash,
	ListChecks,
	type LucideIcon,
	Phone,
	Type,
} from 'lucide-react'
import type { QuestionKind } from '#/pages/survey/types'

export interface CatalogueEntry {
	icon: LucideIcon
	title: string
	description: string
	hue: number
}

export interface QuestionKindEntry extends CatalogueEntry {
	kind: QuestionKind
}

export const QUESTION_KINDS: readonly QuestionKindEntry[] = [
	{
		kind: 'TEXT',
		icon: Type,
		title: 'Texte court',
		description: 'Une réponse en une ligne : un nom, une ville, une référence.',
		hue: 220,
	},
	{
		kind: 'LONG_TEXT',
		icon: AlignLeft,
		title: 'Texte long',
		description: 'Un paragraphe libre pour un commentaire ou une précision.',
		hue: 230,
	},
	{
		kind: 'NUMBER',
		icon: Hash,
		title: 'Nombre',
		description: 'Une valeur chiffrée : une quantité, une surface, une note.',
		hue: 260,
	},
	{
		kind: 'SINGLE_CHOICE',
		icon: CircleDot,
		title: 'Choix unique',
		description: 'Une seule option à cocher parmi celles que vous proposez.',
		hue: 150,
	},
	{
		kind: 'MULTIPLE_CHOICE',
		icon: ListChecks,
		title: 'Choix multiple',
		description: 'Plusieurs options à cocher parmi celles que vous proposez.',
		hue: 165,
	},
	{
		kind: 'DATE',
		icon: Calendar,
		title: 'Date',
		description: 'Un jour du calendrier, sans heure.',
		hue: 30,
	},
	{
		kind: 'DATETIME',
		icon: CalendarClock,
		title: 'Date et heure',
		description: 'Un jour et une heure précise, pour un rendez-vous.',
		hue: 45,
	},
	{
		kind: 'PHONE',
		icon: Phone,
		title: 'Téléphone',
		description: 'Un numéro de téléphone vérifié à la saisie.',
		hue: 195,
	},
	{
		kind: 'EMAIL',
		icon: AtSign,
		title: 'E-mail',
		description: 'Une adresse e-mail vérifiée à la saisie.',
		hue: 205,
	},
	{
		kind: 'PHOTO',
		icon: Camera,
		title: 'Photo',
		description: 'Une ou plusieurs photos prises ou déposées par le répondant.',
		hue: 330,
	},
]

const BY_KIND = new Map(QUESTION_KINDS.map((entry) => [entry.kind, entry]))

export function questionKindEntry(kind: QuestionKind): QuestionKindEntry {
	const entry = BY_KIND.get(kind)
	if (!entry) throw new Error(`question-kinds: ${kind} manquant`)
	return entry
}

export function isChoiceKind(kind: QuestionKind): boolean {
	return kind === 'SINGLE_CHOICE' || kind === 'MULTIPLE_CHOICE'
}

export function isIdentifierKind(kind: QuestionKind): boolean {
	return kind === 'TEXT' || kind === 'EMAIL' || kind === 'PHONE'
}
