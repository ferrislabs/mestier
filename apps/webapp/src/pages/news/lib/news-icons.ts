import {
	BarChart3,
	Bell,
	CalendarDays,
	FileText,
	ImageIcon,
	type LucideIcon,
	Map as MapIcon,
	MessageSquare,
	Plug,
	Rocket,
	Settings,
	Shield,
	Sparkles,
	Star,
	Users,
	Wrench,
	Zap,
} from 'lucide-react'
import type { Schemas } from '#/api/api.client'

export type NewsIconName = Schemas.NewsIcon

const ICONS: Record<NewsIconName, LucideIcon> = {
	plug: Plug,
	sparkles: Sparkles,
	rocket: Rocket,
	chart: BarChart3,
	bell: Bell,
	calendar: CalendarDays,
	file: FileText,
	users: Users,
	shield: Shield,
	zap: Zap,
	image: ImageIcon,
	message: MessageSquare,
	settings: Settings,
	map: MapIcon,
	wrench: Wrench,
	star: Star,
}

const LABELS: Record<NewsIconName, string> = {
	plug: 'Connexion',
	sparkles: 'Nouveauté',
	rocket: 'Lancement',
	chart: 'Statistiques',
	bell: 'Notification',
	calendar: 'Calendrier',
	file: 'Document',
	users: 'Équipe',
	shield: 'Sécurité',
	zap: 'Rapidité',
	image: 'Image',
	message: 'Message',
	settings: 'Réglages',
	map: 'Carte',
	wrench: 'Outil',
	star: 'Favori',
}

export const NEWS_ICON_OPTIONS: ReadonlyArray<{
	value: NewsIconName
	label: string
}> = (Object.keys(ICONS) as NewsIconName[]).map((value) => ({
	value,
	label: LABELS[value],
}))

export function newsIconComponent(name: NewsIconName): LucideIcon {
	return ICONS[name] ?? Sparkles
}
