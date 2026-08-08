import type { LucideIcon } from 'lucide-react'

export type ModuleId = 'home' | 'crm' | 'hr' | 'planning' | 'chat' | 'settings'

/**
 * `coming-soon` : module annoncé mais non navigable — il reste visible, grisé,
 * dans le sélecteur de modules.
 * `hidden` : jamais rendu. Réservé au futur filtrage par droits, hors scope.
 */
export type ModuleStatus = 'available' | 'coming-soon' | 'hidden'

export type NavStatus = Exclude<ModuleStatus, 'hidden'>

/** Cible de navigation, commune aux deux niveaux. */
export interface NavTarget {
	id: string
	label: string
	to: string
	icon?: LucideIcon
	/** Actif uniquement sur une correspondance exacte du chemin. */
	exact?: boolean
	status?: NavStatus
	badge?: string | number
}

/** Second niveau : onglets horizontaux, sous le fil d'Ariane. */
export type ModuleTab = NavTarget

/** Premier niveau : entrées de la nav latérale du module. */
export interface ModuleSection extends NavTarget {
	tabs?: ModuleTab[]
}

export interface AppModule {
	id: ModuleId
	label: string
	icon: LucideIcon
	basePath: string
	status: ModuleStatus
	/** Navigation intra-module, rendue dans la colonne de gauche. */
	sections: ModuleSection[]
	/**
	 * `basePath` rend une vue d'ensemble au lieu de rediriger vers la première
	 * section navigable.
	 */
	hasOverview: boolean
	/**
	 * `utility` place le module en pied de nav, séparé des modules métier.
	 * Par défaut : `primary`.
	 */
	railPlacement?: 'primary' | 'utility'
}
