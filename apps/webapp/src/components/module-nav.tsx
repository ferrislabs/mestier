import { Link, useLocation } from '@tanstack/react-router'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
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
	useSidebar,
} from '#/components/ui/sidebar'
import { usePermissions } from '#/hooks/use-permissions'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import { resolveModule } from '#/modules/resolve-module'
import type { AppModule, NavTarget } from '#/modules/types'

interface ModuleNavProps extends React.ComponentProps<typeof Sidebar> {
	organizationSlug: string
	/** Account menu, rendered at the foot of the nav. */
	footer?: React.ReactNode
}

const itemClassName =
	'h-8 gap-2.5 text-[13px] text-nav-muted hover:bg-nav-active hover:text-nav-foreground data-[active=true]:bg-nav-active data-[active=true]:font-medium data-[active=true]:text-nav-active-foreground'

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
	const { state, toggleSidebar } = useSidebar()
	const activeModule = resolveModule(splitOrgPath(location.pathname).path)
	const utilityModules = MODULES.filter(
		(module) =>
			module.status !== 'hidden' && module.railPlacement === 'utility',
	)

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

			<SidebarContent className="px-2 pt-1">
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
				{activeModule.sidebarExtra ? (
					<div className="group-data-[collapsible=icon]:hidden">
						<activeModule.sidebarExtra />
					</div>
				) : null}
			</SidebarContent>

			<SidebarFooter className="gap-1 border-nav-border border-t p-2">
				<SidebarMenu className="gap-px">
					{utilityModules.map((module) => (
						<UtilityEntry
							key={module.id}
							module={module}
							active={module.id === activeModule.id}
							organizationSlug={organizationSlug}
						/>
					))}
					<SidebarMenuItem>
						<SidebarMenuButton
							onClick={toggleSidebar}
							tooltip={state === 'expanded' ? 'Réduire' : 'Agrandir'}
							className={itemClassName}
						>
							{state === 'expanded' ? <PanelLeftClose /> : <PanelLeftOpen />}
							<span>{state === 'expanded' ? 'Réduire' : 'Agrandir'}</span>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
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
					<Icon />
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
					<Icon />
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

interface UtilityEntryProps {
	module: AppModule
	active: boolean
	organizationSlug: string
}

function UtilityEntry({ module, active, organizationSlug }: UtilityEntryProps) {
	const Icon = module.icon

	return (
		<SidebarMenuItem>
			<SidebarMenuButton
				asChild
				isActive={active}
				tooltip={module.label}
				className={itemClassName}
			>
				<Link to={buildOrgPath(organizationSlug, module.basePath)}>
					<Icon />
					<span>{module.label}</span>
				</Link>
			</SidebarMenuButton>
		</SidebarMenuItem>
	)
}
