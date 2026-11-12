import type { LucideIcon } from 'lucide-react'
import { Building2, LayoutDashboard, Megaphone, Users } from 'lucide-react'

export interface AdminSection {
	id: string
	label: string
	to: '/admin' | '/admin/organisations' | '/admin/utilisateurs' | '/admin/news'
	icon: LucideIcon
	exact?: boolean
}

export const ADMIN_SECTIONS: readonly AdminSection[] = [
	{
		id: 'overview',
		label: 'Vue d’ensemble',
		to: '/admin',
		icon: LayoutDashboard,
		exact: true,
	},
	{
		id: 'organizations',
		label: 'Organisations',
		to: '/admin/organisations',
		icon: Building2,
	},
	{
		id: 'users',
		label: 'Utilisateurs',
		to: '/admin/utilisateurs',
		icon: Users,
	},
	{ id: 'news', label: 'News', to: '/admin/news', icon: Megaphone },
]

export type AdminGuard = 'loading' | 'allowed' | 'denied'

export function adminGuard(
	isPending: boolean,
	canAccess: boolean | undefined,
): AdminGuard {
	if (isPending) return 'loading'
	return canAccess === true ? 'allowed' : 'denied'
}
