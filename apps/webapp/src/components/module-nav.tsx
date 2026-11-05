import { Link, useLocation } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import type * as React from 'react'

import { ModuleIcon } from '#/components/module-icon'
import { ModuleLauncher } from '#/components/module-launcher'
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
} from '#/components/ui/sidebar'
import { usePermissions } from '#/hooks/use-permissions'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'
import type { AppModule, NavTarget } from '#/modules/types'

interface ModuleNavProps extends React.ComponentProps<typeof Sidebar> {
	organizationSlug: string
	/** Account menu, rendered at the foot of the nav. */
	footer?: React.ReactNode
}

const itemClassName =
	'h-8 gap-2.5 px-2.5 text-[13px] text-nav-muted hover:bg-nav-active hover:text-nav-foreground data-[active=true]:bg-nav-active data-[active=true]:font-medium data-[active=true]:text-nav-active-foreground'

/**
 * Navigation for the active module, in the left column.
 *
 * It is not for switching modules — each module is its own app, the way
 * Calendar and Docs each own their nav rather than sharing a cross-app rail;
 * switching is the {@link ModuleLauncher}'s job, which sits in this column's
 * header next to the module's own mark. It lists the current module's screens;
 * their second level, when they have one, is rendered as horizontal tabs by the
 * {@link ScopeBar}.
 */
export function ModuleNav({
	organizationSlug,
	footer,
	className,
	...props
}: ModuleNavProps) {
	const location = useLocation()
	const activeModule = resolveModule(splitOrgPath(location.pathname).path)

	// One read for every section's `requiredPermission` at once, rather than
	// a `useHasPermission` call per section: that would call a hook a
	// varying number of times across renders, which React disallows.
	// Absent while the read is still loading — a section gated by a bit is
	// hidden by default, the same call `usePermissions` itself makes.
	const { data: permissionsData, isSuccess: permissionsLoaded } =
		usePermissions()
	const grantedPermissions = permissionsData?.data.permissions ?? []
	const visibleSections = activeModule.sections.filter(
		(section) =>
			!section.requiredPermission ||
			(permissionsLoaded &&
				grantedPermissions.includes(section.requiredPermission)),
	)

	const sections = (
		<SidebarGroup className="p-0">
			<SidebarMenu className="gap-px">
				{visibleSections.map((section) => (
					<NavEntry
						key={section.id}
						target={section}
						organizationSlug={organizationSlug}
						fallbackIcon={activeModule.icon}
					/>
				))}
			</SidebarMenu>
		</SidebarGroup>
	)

	return (
		<Sidebar collapsible="icon" className={className} {...props}>
			<SidebarHeader className="h-12 flex-row items-center gap-2 p-0 pr-2 pl-3 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
				<Link
					to={buildOrgPath(organizationSlug, activeModule.basePath)}
					// The module's own mark is a way back to its entry, not one of its
					// sections: only the section matching the route is `aria-current`.
					activeOptions={{ exact: true }}
					activeProps={{ 'aria-current': undefined }}
					className="flex min-w-0 items-center gap-2 rounded-md font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring"
				>
					<ModuleIcon module={activeModule} size="sm" />
					<span className="truncate text-[15px] group-data-[collapsible=icon]:hidden">
						{activeModule.label}
					</span>
				</Link>
				<ModuleLauncher
					activeModuleId={activeModule.id}
					organizationSlug={organizationSlug}
					className="ml-auto text-nav-muted hover:bg-nav-active hover:text-nav-foreground group-data-[collapsible=icon]:hidden"
				/>
			</SidebarHeader>

			{activeModule.primaryAction ? (
				<div className="px-3 pt-1 pb-3 group-data-[collapsible=icon]:hidden">
					<Link
						to={buildOrgPath(organizationSlug, activeModule.primaryAction.to)}
						search={activeModule.primaryAction.search}
						className="flex h-9 min-w-0 items-center justify-center gap-1.5 whitespace-nowrap rounded-md bg-nav-primary px-3 font-medium text-[13px] text-nav-primary-foreground outline-none transition-opacity hover:opacity-90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-nav"
					>
						<Plus className="size-4" strokeWidth={2} aria-hidden />
						<span className="truncate">{activeModule.primaryAction.label}</span>
					</Link>
				</div>
			) : null}

			<SidebarContent className="px-2 pt-1 pb-2">
				{activeModule.sidebar ? (
					<>
						<div className="group-data-[collapsible=icon]:hidden">
							<activeModule.sidebar />
						</div>
						<div className="hidden group-data-[collapsible=icon]:block">
							{sections}
						</div>
					</>
				) : (
					sections
				)}
				{activeModule.sidebarExtra ? (
					<div className="group-data-[collapsible=icon]:hidden">
						<activeModule.sidebarExtra />
					</div>
				) : null}
			</SidebarContent>

			<SidebarFooter className="border-nav-border border-t p-2">
				{footer}
			</SidebarFooter>
		</Sidebar>
	)
}

interface NavEntryProps {
	target: NavTarget
	organizationSlug: string
	fallbackIcon: AppModule['icon']
}

function NavEntry({ target, organizationSlug, fallbackIcon }: NavEntryProps) {
	const Icon = target.icon ?? fallbackIcon

	if (target.status === 'coming-soon') {
		return (
			<SidebarMenuItem>
				<SidebarMenuButton
					aria-disabled
					tooltip={`${target.label} · bientôt`}
					className={itemClassName}
				>
					<Icon strokeWidth={1.8} />
					<span>{target.label}</span>
					<span className="ml-auto rounded-md border border-nav-border px-1.5 py-0.5 text-[10px] font-medium group-data-[collapsible=icon]:hidden">
						bientôt
					</span>
				</SidebarMenuButton>
			</SidebarMenuItem>
		)
	}

	return (
		<SidebarMenuItem>
			<SidebarMenuButton
				asChild
				tooltip={target.label}
				className={itemClassName}
			>
				<Link
					to={buildOrgPath(organizationSlug, target.to)}
					activeOptions={target.exact ? { exact: true } : undefined}
					activeProps={{ 'data-active': 'true', 'aria-current': 'page' }}
				>
					<Icon strokeWidth={1.8} />
					<span>{target.label}</span>
					{target.badge !== undefined ? (
						<span className="tabular ml-auto text-[11px] text-nav-muted group-data-[collapsible=icon]:hidden">
							{target.badge}
						</span>
					) : null}
				</Link>
			</SidebarMenuButton>
		</SidebarMenuItem>
	)
}
