import { Link } from '@tanstack/react-router'

import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { ModuleTab } from '#/modules/types'

interface NavTabsProps {
	/** Describes the navigated scope, for screen readers. */
	label: string
	tabs: ModuleTab[]
	/** Tenant the links hang off: tabs are declared relative. */
	organizationSlug: string
}

// Material indicator: a thick rounded line under the active tab, not a solid
// background or a container border.
const tabClassName =
	'relative inline-flex items-center gap-1.5 whitespace-nowrap rounded-t-lg px-4 py-3.5 text-sm font-medium transition-colors after:absolute after:inset-x-0 after:bottom-0 after:h-[3px] after:rounded-t-full after:bg-primary after:opacity-0 after:transition-opacity'

/**
 * Navigation tabs for a scope.
 *
 * These are not Radix's `Tabs`: a navigation tab is a link that changes route,
 * not a trigger revealing a local `TabsContent`. Borrowing the component would
 * force an `aria-controls` pointing at a panel that does not exist, and a
 * roving focus competing with native keyboard navigation. So we take the visual
 * shape of the `line` variant inside a `<nav>` of links.
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
