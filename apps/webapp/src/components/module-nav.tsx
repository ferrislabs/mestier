import { Link, useLocation } from '@tanstack/react-router'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import type * as React from 'react'

import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	useSidebar,
} from '#/components/ui/sidebar'
import { usePermissions } from '#/hooks/use-permissions'
import { cn } from '#/lib/utils'
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
 * It is not for switching modules — each module is its own app, the way
 * Calendar and Docs each own their nav rather than sharing a cross-app rail;
 * switching is the header {@link ModuleLauncher}'s job. It lists the current
 * module's screens; their second level, when they have one, is rendered as
 * horizontal tabs by the {@link ScopeBar}. No brand header of its own —
 * that lives in {@link AppHeader} now, common to every module.
 */
export function ModuleNav({
	organizationSlug,
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
		<Sidebar
			collapsible="icon"
			{...props}
			// `AppHeader` now spans the full width above this nav (see
			// `_app.o.$organizationSlug.tsx`) rather than beside it — the
			// sidebar's own fixed container defaults to the full viewport
			// height (`inset-y-0 h-svh`); overridden (`!`, `inset-y-0`'s own
			// utility would otherwise win) so it starts below the header
			// instead of underneath it.
			className={cn(
				'top-(--app-header-height)! h-[calc(100svh-var(--app-header-height))]!',
				className,
			)}
		>
			<SidebarContent className="pt-3">
				<SidebarGroup>
					<SidebarMenu>
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
					{/* Replaces the old edge-drag rail — a 4px invisible hitbox,
					 * `tabIndex={-1}`, discoverable only by accident. Same toggle,
					 * as an actual button instead of a hidden one. */}
					<SidebarMenuItem>
						<SidebarMenuButton
							onClick={toggleSidebar}
							tooltip={state === 'expanded' ? 'Réduire' : 'Agrandir'}
							className={pillClassName}
						>
							{state === 'expanded' ? <PanelLeftClose /> : <PanelLeftOpen />}
							<span>{state === 'expanded' ? 'Réduire' : 'Agrandir'}</span>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
			</SidebarFooter>
		</Sidebar>
	)
}

// Sharp-cornered, not a pill: a left rule was tried first, but padding
// between the button and the sidebar's own edge kept it from ever sitting
// flush — reading as a misaligned accent rather than a deliberate one. A
// background fill has no edge to miss.
const pillClassName =
	'rounded-none! font-medium ' +
	'hover:bg-muted! ' +
	'data-[active=true]:bg-brand-soft! data-[active=true]:font-semibold data-[active=true]:text-brand-muted ' +
	'group-data-[collapsible=icon]:rounded-none!'

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
