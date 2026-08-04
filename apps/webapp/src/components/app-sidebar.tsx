import { Link, useLocation } from '@tanstack/react-router'
import type * as React from 'react'

import { MestierAppIcon } from '#/components/brand/mestier-logo'
import { NavMain } from '#/components/nav-main'
import { TeamSwitcher } from '#/components/team-switcher'
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarRail,
} from '#/components/ui/sidebar'
import { buildSidebarGroups } from '#/modules/nav-groups'

export function AppSidebar(props: React.ComponentProps<typeof Sidebar>) {
	const location = useLocation()
	const groups = buildSidebarGroups(location.pathname)

	return (
		<Sidebar collapsible="icon" {...props}>
			<SidebarHeader className="gap-3 border-b border-sidebar-border/70 pb-3">
				<SidebarMenu>
					<SidebarMenuItem>
						<SidebarMenuButton size="lg" asChild tooltip="Mestier">
							<Link to="/">
								<MestierAppIcon className="size-9 border border-sidebar-border bg-white/10 text-white shadow-none" />
								<div className="grid flex-1 text-left leading-tight">
									<span className="truncate text-base font-semibold text-sidebar-foreground">
										Mestier
									</span>
									<span className="truncate text-[10px] font-medium uppercase tracking-wider text-white/60">
										Console
									</span>
								</div>
							</Link>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
				<TeamSwitcher />
			</SidebarHeader>
			<SidebarContent>
				{groups.map((group, index) => (
					<NavMain
						key={group.label ?? `group-${index}`}
						label={group.label}
						items={group.items}
					/>
				))}
			</SidebarContent>
			<SidebarFooter>
				<div className="flex items-center justify-start px-2 pb-1 text-[10px] font-medium group-data-[collapsible=icon]:hidden">
					<span className="rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 text-white/70">
						0.1.0
					</span>
				</div>
			</SidebarFooter>
			<SidebarRail />
		</Sidebar>
	)
}
