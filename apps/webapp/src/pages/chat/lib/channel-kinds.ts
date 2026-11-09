import {
	FolderTree,
	Hash,
	type LucideIcon,
	Megaphone,
	Volume2,
} from 'lucide-react'

export type ChannelKind = 'TEXT' | 'ANNOUNCEMENT' | 'VOICE'

/** What the create dialog can make: a channel of one of three kinds, or a
 * category that holds channels. */
export type NewItemKind = ChannelKind | 'CATEGORY'

export interface ChannelKindOption<T extends string = ChannelKind> {
	value: T
	title: string
	description: string
	icon: LucideIcon
}

export const CHANNEL_KINDS: readonly ChannelKindOption[] = [
	{
		value: 'TEXT',
		title: 'Texte',
		description: 'Des messages, des fichiers et des fils pour discuter.',
		icon: Hash,
	},
	{
		value: 'ANNOUNCEMENT',
		title: 'Annonces',
		description: 'Seuls les gestionnaires de canaux publient, tous lisent.',
		icon: Megaphone,
	},
	{
		value: 'VOICE',
		title: 'Vocal',
		description: 'Un salon pour parler à voix haute. Bientôt disponible.',
		icon: Volume2,
	},
]

export const CATEGORY_KIND: ChannelKindOption<'CATEGORY'> = {
	value: 'CATEGORY',
	title: 'Catégorie',
	description: 'Un dossier pour regrouper des canaux dans la barre latérale.',
	icon: FolderTree,
}

/** The cards of the create dialog, category last. */
export const NEW_ITEM_KINDS: readonly ChannelKindOption<NewItemKind>[] = [
	...CHANNEL_KINDS,
	CATEGORY_KIND,
]

/** The icon in front of a channel's name, in the side nav and the header. */
export function channelKindIcon(type: string | undefined): LucideIcon {
	return CHANNEL_KINDS.find((kind) => kind.value === type)?.icon ?? Hash
}
