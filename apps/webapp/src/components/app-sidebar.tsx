import { Link } from '@tanstack/react-router'
import {
	BarChart3,
	Eye,
	FileText,
	LayoutDashboard,
	Link2,
	Package,
	Receipt,
	Settings,
	ShieldCheck,
	Users,
} from 'lucide-react'
import type * as React from 'react'

import { MestierAppIcon } from '#/components/brand/mestier-logo'
import { type NavItem, NavMain } from '#/components/nav-main'
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

const coreItems: NavItem[] = [
	{ title: 'Accueil', to: '/', icon: LayoutDashboard, exact: true },
	{ title: 'Clients', to: '/customers', icon: Users, badge: '3' },
	{ title: 'Devis', to: '/customers', icon: FileText, disabled: true },
	{ title: 'Factures', to: '/customers', icon: Receipt, disabled: true },
	{ title: 'Stock', to: '/customers', icon: Package, disabled: true },
]

const configItems: NavItem[] = [
	{ title: 'Paramètres', to: '/settings', icon: Settings },
	{ title: 'Intégrations', to: '/customers', icon: Link2, disabled: true },
	{ title: 'Rapports', to: '/customers', icon: BarChart3, disabled: true },
]

const securityItems: NavItem[] = [
	{ title: 'Utilisateurs', to: '/users', icon: Users },
	{ title: 'Audit', to: '/customers', icon: Eye, disabled: true },
	{
		title: 'Permissions',
		to: '/customers',
		icon: ShieldCheck,
		disabled: true,
	},
]

export function AppSidebar(props: React.ComponentProps<typeof Sidebar>) {
	return (
		<Sidebar collapsible="icon" {...props}>
			<SidebarHeader className="border-b border-sidebar-border/70 pb-3">
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
			</SidebarHeader>
			<SidebarContent>
				<NavMain label="Activité" items={coreItems} />
				<NavMain label="Configuration" items={configItems} />
				<NavMain label="Sécurité" items={securityItems} />
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
