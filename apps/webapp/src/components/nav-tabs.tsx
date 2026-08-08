import { Link } from '@tanstack/react-router'

import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { ModuleTab } from '#/modules/types'

interface NavTabsProps {
	/** Décrit le scope navigué, pour les lecteurs d'écran. */
	label: string
	tabs: ModuleTab[]
	/** Tenant dont dépendent les liens : les onglets sont déclarés relatifs. */
	organizationSlug: string
}

// Indicateur Material : un trait épais arrondi sous l'onglet actif, et non un
// fond plein ou une bordure de conteneur.
const tabClassName =
	'relative inline-flex items-center gap-1.5 whitespace-nowrap rounded-t-lg px-4 py-3.5 text-sm font-medium transition-colors after:absolute after:inset-x-0 after:bottom-0 after:h-[3px] after:rounded-t-full after:bg-primary after:opacity-0 after:transition-opacity'

/**
 * Onglets de navigation d'un scope.
 *
 * Ce ne sont pas les `Tabs` de Radix : un onglet de navigation est un lien qui
 * change de route, pas un déclencheur qui révèle un `TabsContent` local. Emprunter
 * le composant imposerait un `aria-controls` pointant vers un panneau inexistant
 * et un focus roving concurrent de la navigation clavier native. On reprend donc
 * la forme visuelle de la variante `line` dans un `<nav>` de liens.
 */
export function NavTabs({ label, tabs, organizationSlug }: NavTabsProps) {
	return (
		<nav aria-label={label} className="min-w-0 flex-1">
			<ul className="flex items-center gap-1 overflow-x-auto">
				{tabs.map((tab) => (
					<li key={tab.id} className="shrink-0">
						{tab.status === 'coming-soon' ? (
							<button
								type="button"
								aria-disabled="true"
								className={cn(
									tabClassName,
									'cursor-not-allowed text-muted-foreground/70',
								)}
							>
								{tab.icon ? <tab.icon className="size-4" /> : null}
								<span>{tab.label}</span>
								<span className="rounded-md border px-1.5 py-0.5 text-[10px] font-medium">
									bientôt
								</span>
							</button>
						) : (
							<Link
								to={buildOrgPath(organizationSlug, tab.to)}
								activeOptions={tab.exact ? { exact: true } : undefined}
								activeProps={{
									'data-active': 'true',
									'aria-current': 'page',
								}}
								className={cn(
									tabClassName,
									'text-muted-foreground hover:bg-muted/60 hover:text-foreground data-[active=true]:text-primary data-[active=true]:after:opacity-100',
								)}
							>
								{tab.icon ? <tab.icon className="size-4" /> : null}
								<span>{tab.label}</span>
								{tab.badge !== undefined ? (
									<span className="rounded-md bg-muted px-1.5 py-0.5 text-xs font-semibold text-muted-foreground">
										{tab.badge}
									</span>
								) : null}
							</Link>
						)}
					</li>
				))}
			</ul>
		</nav>
	)
}
