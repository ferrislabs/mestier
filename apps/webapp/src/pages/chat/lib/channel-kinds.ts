import { Hash, type LucideIcon, Megaphone, Volume2 } from 'lucide-react'

export type ChannelKind = 'TEXT' | 'ANNOUNCEMENT' | 'VOICE'

export interface ChannelKindOption {
	value: ChannelKind
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

/** The icon in front of a channel's name, in the side nav and the header. */
export function channelKindIcon(type: string | undefined): LucideIcon {
	return CHANNEL_KINDS.find((kind) => kind.value === type)?.icon ?? Hash
}
