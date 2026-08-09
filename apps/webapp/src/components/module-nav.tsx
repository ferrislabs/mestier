import { Link, useLocation } from '@tanstack/react-router'
import type * as React from 'react'

import { MestierAppIcon } from '#/components/brand/mestier-logo'
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarRail as SidebarResizeHandle,
} from '#/components/ui/sidebar'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import { resolveModule } from '#/modules/resolve-module'
import type { AppModule, NavTarget } from '#/modules/types'

interface ModuleNavProps extends React.ComponentProps<typeof Sidebar> {
	organizationSlug: string
}

/**
 * Navigation for the active module, in the left column.
 *
 * It is not for switching modules — that is the header {@link ModuleLauncher}'s
 * job. It lists the current module's screens; their second level, when they
 * have one, is rendered as horizontal tabs by the {@link ScopeBar}.
 */
export function ModuleNav({ organizationSlug, ...props }: ModuleNavProps) {
	const location = useLocation()
	const activeModule = resolveModule(splitOrgPath(location.pathname).path)
	const utilityModules = MODULES.filter(
		(module) =>
			module.status !== 'hidden' && module.railPlacement === 'utility',
	)

	return (
		<Sidebar collapsible="icon" {...props}>
			<SidebarHeader className="pb-3">
				<div className="flex items-center gap-2 px-1 py-1">
					<MestierAppIcon className="size-8 shrink-0 rounded-lg" />
					<div className="grid min-w-0 flex-1 leading-tight group-data-[collapsible=icon]:hidden">
						<span className="truncate text-base font-medium text-sidebar-foreground">
							Mestier
						</span>
						<span className="truncate text-xs text-sidebar-foreground/60">
							{activeModule.label}
						</span>
					</div>
				</div>
			</SidebarHeader>

			<SidebarContent>
				<SidebarGroup>
					<SidebarMenu>
						{activeModule.sections.map((section) => (
							<NavEntry
								key={section.id}
								target={section}
								organizationSlug={organizationSlug}
								fallbackIcon={activeModule.icon}
							/>
						))}
					</SidebarMenu>
				</SidebarGroup>
			</SidebarContent>

			<SidebarFooter>
				<SidebarMenu>
					{utilityModules.map((module) => (
						<UtilityEntry
							key={module.id}
							module={module}
							active={module.id === activeModule.id}
							organizationSlug={organizationSlug}
						/>
					))}
				</SidebarMenu>
			</SidebarFooter>

			<SidebarResizeHandle />
		</Sidebar>
	)
}

// Rounded pill, like the Google consoles' nav: the active state is a brand
// tint, not a background contrast.
const pillClassName =
	'rounded-full font-medium data-[active=true]:font-semibold group-data-[collapsible=icon]:rounded-full'

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
					className={pillClassName}
				>
					<Icon />
					<span>{target.label}</span>
					<span className="ml-auto rounded-md border border-sidebar-border px-1.5 py-0.5 text-[10px] font-medium group-data-[collapsible=icon]:hidden">
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
				className={pillClassName}
			>
				<Link
					to={buildOrgPath(organizationSlug, target.to)}
					activeOptions={target.exact ? { exact: true } : undefined}
					activeProps={{ 'data-active': 'true', 'aria-current': 'page' }}
				>
					<Icon />
					<span>{target.label}</span>
					{target.badge !== undefined ? (
						<span className="ml-auto rounded-md bg-sidebar-accent px-1.5 py-0.5 text-xs font-semibold group-data-[collapsible=icon]:hidden">
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
				className={pillClassName}
			>
				<Link to={buildOrgPath(organizationSlug, module.basePath)}>
					<Icon />
					<span>{module.label}</span>
				</Link>
			</SidebarMenuButton>
		</SidebarMenuItem>
	)
}
