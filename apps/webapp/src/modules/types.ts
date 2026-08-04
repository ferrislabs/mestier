import type { LucideIcon } from 'lucide-react'
import type { NavItem } from '#/components/nav-main'

export type ModuleId = 'home' | 'crm' | 'hr' | 'discussions'

export interface ModuleNavGroup {
	label?: string
	items: NavItem[]
}

export interface AppModule {
	id: ModuleId
	label: string
	icon: LucideIcon
	basePath: string
	enabled: boolean
	nav: ModuleNavGroup[]
}
