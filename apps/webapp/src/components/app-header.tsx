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
import { hasSideNav, resolveModule } from '#/modules/resolve-module'
import type { ModuleId } from '#/modules/types'

interface HomeBrandProps {
	organizationSlug: string
	activeModuleId: ModuleId
}

function HomeBrand({ organizationSlug, activeModuleId }: HomeBrandProps) {
	return (
		<span className="flex items-center gap-1">
			<Link
				to={buildOrgPath(organizationSlug, '/')}
				aria-label="Accueil"
				className="flex items-center gap-2 rounded-md py-1 pr-1 pl-0.5 font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring"
			>
				<MestierAppIcon className="size-7 shrink-0" />
				<span className="text-[15px]">Mestier</span>
			</Link>
			<ModuleLauncher
				activeModuleId={activeModuleId}
				organizationSlug={organizationSlug}
				className="text-muted-foreground hover:bg-accent hover:text-foreground"
			/>
		</span>
	)
}

/**
 * Header of the content column. On a module screen the module's own mark and
 * the launcher live at the top of the side nav; on a screen without side nav
 * (the homepage, the settings) they move here next to the brand, together with
 * the account menu.
 */
export function AppHeader() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const path = splitOrgPath(location.pathname).path
	const activeModule = resolveModule(path)
	const withSideNav = hasSideNav(activeModule, path)

	return (
		<header className="flex h-12 shrink-0 items-center gap-3 border-b bg-background px-3">
			{withSideNav ? (
				<>
					<SidebarTrigger className="md:hidden" />
					<ModuleLauncher
						activeModuleId={activeModule.id}
						organizationSlug={activeOrganization.slug}
						className="text-muted-foreground hover:bg-accent md:hidden"
					/>
				</>
			) : null}

			<AppBreadcrumb
				leading={
					withSideNav ? undefined : (
						<HomeBrand
							organizationSlug={activeOrganization.slug}
							activeModuleId={activeModule.id}
						/>
					)
				}
			/>

			<div className="ml-auto flex items-center gap-1.5">
				<SearchCommand
					organizationSlug={activeOrganization.slug}
					className="w-60 border bg-card text-muted-foreground hover:text-foreground max-lg:w-8 max-lg:justify-center max-lg:px-0 max-lg:[&>kbd]:hidden max-lg:[&>span]:hidden"
				/>
				<button
					type="button"
					aria-label="Notifications"
					className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground outline-none hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
				>
					<Bell className="size-4" strokeWidth={1.8} />
				</button>
				{withSideNav ? null : <UserMenu />}
			</div>
		</header>
	)
}
