import { Link, useLocation } from '@tanstack/react-router'
import { Bell } from 'lucide-react'

import { AppBreadcrumb } from '#/components/app-breadcrumb'
import { MestierAppIcon } from '#/components/brand/mestier-logo'
import { ModuleLauncher } from '#/components/module-launcher'
import { SearchCommand } from '#/components/search-command'
import { SidebarTrigger } from '#/components/ui/sidebar'
import { UserMenu } from '#/components/user-menu'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'

/**
 * Header of the content column. On a module screen the module's own mark and
 * the launcher live at the top of the side nav; on the homepage, which has no
 * side nav, they move here next to the brand.
 */
export function AppHeader() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const path = splitOrgPath(location.pathname).path
	const activeModule = resolveModule(path)
	const isHome = path === '/'

	return (
		<header className="flex h-(--app-header-height) shrink-0 items-center gap-3 border-b bg-background px-3">
			{isHome ? (
				<span className="flex items-center gap-1">
					<Link
						to={buildOrgPath(activeOrganization.slug, '/')}
						aria-label="Accueil"
						className="flex items-center gap-2 rounded-md py-1 pr-1 pl-0.5 font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						<MestierAppIcon className="size-7 shrink-0" />
						<span className="text-[15px]">Mestier</span>
					</Link>
					<ModuleLauncher
						activeModuleId={activeModule.id}
						organizationSlug={activeOrganization.slug}
						className="text-muted-foreground hover:bg-accent hover:text-foreground"
					/>
				</span>
			) : (
				// On mobile the side nav is a panel: it needs a trigger.
				<SidebarTrigger className="md:hidden" />
			)}

			<AppBreadcrumb />

			<div className="ml-auto flex items-center gap-1.5">
				<SearchCommand
					organizationSlug={activeOrganization.slug}
					className="w-60 border bg-card text-muted-foreground hover:text-foreground max-lg:w-8 max-lg:justify-center max-lg:px-0 max-lg:[&>kbd]:hidden max-lg:[&>span]:hidden"
				/>
				{/*
				 * A deliberate placeholder: no event stream is wired up yet. The day
				 * one is, the counter will come from the realtime gateway — this
				 * button must not be turned into periodic polling.
				 */}
				<button
					type="button"
					aria-label="Notifications"
					className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground outline-none hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
				>
					<Bell className="size-4" strokeWidth={1.8} />
				</button>
				{isHome ? (
					<UserMenu className="text-foreground" />
				) : (
					<UserMenu className="text-foreground md:hidden" />
				)}
			</div>
		</header>
	)
}
