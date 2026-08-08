import type * as React from 'react'

import { NavTabs } from '#/components/nav-tabs'
import type { ModuleTab } from '#/modules/types'

interface ScopeBarProps {
	label: string
	tabs: ModuleTab[]
	organizationSlug: string
	/**
	 * Actions du scope courant — sauvegarde d'une fiche, création d'une entité.
	 * Elles vivent ici plutôt que dans une barre flottante au-dessus du contenu.
	 */
	actions?: React.ReactNode
}

/**
 * Barre de navigation du scope actif, collée sous l'en-tête.
 *
 * Un scope qui n'expose qu'un seul écran n'affiche pas de barre : le rail et le
 * fil d'Ariane suffisent à situer l'utilisateur.
 */
export function ScopeBar({
	label,
	tabs,
	organizationSlug,
	actions,
}: ScopeBarProps) {
	if (tabs.length <= 1 && !actions) return null

	return (
		<div className="sticky top-(--app-header-height) z-10 flex items-center gap-4 border-b bg-card px-3 md:px-6">
			<NavTabs label={label} tabs={tabs} organizationSlug={organizationSlug} />
			{actions ? (
				<div className="flex shrink-0 items-center gap-2 py-2">{actions}</div>
			) : null}
		</div>
	)
}
