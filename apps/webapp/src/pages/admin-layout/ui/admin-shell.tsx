import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import type { ReactNode } from 'react'
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarHeader,
	SidebarInset,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarProvider,
} from '#/components/ui/sidebar'
import { ADMIN_SECTIONS } from '#/pages/admin-layout/lib/admin-sections'

const itemClassName =
	'h-8 gap-2.5 px-2.5 text-[13px] text-nav-muted hover:bg-nav-active hover:text-nav-foreground data-[active=true]:bg-nav-active data-[active=true]:font-medium data-[active=true]:text-nav-active-foreground'

interface AdminShellProps {
	children: ReactNode
}

export function AdminShell({ children }: AdminShellProps) {
	return (
		<SidebarProvider className="h-svh overflow-hidden">
			<Sidebar collapsible="icon" className="border-nav-border bg-sidebar">
				<SidebarHeader className="h-12 flex-row items-center gap-2 p-0 pl-3 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
					<span className="truncate font-semibold text-[15px] group-data-[collapsible=icon]:hidden">
						Back-office
					</span>
				</SidebarHeader>
				<SidebarContent className="gap-0 px-2 pb-2">
					<SidebarGroup className="p-0">
						<SidebarMenu className="gap-px">
							{ADMIN_SECTIONS.map((section) => (
								<SidebarMenuItem key={section.id}>
									<SidebarMenuButton
										asChild
										tooltip={section.label}
										className={itemClassName}
									>
										<Link
											to={section.to}
											activeOptions={
												section.exact ? { exact: true } : undefined
											}
											activeProps={{
												'data-active': 'true',
												'aria-current': 'page',
											}}
										>
											<section.icon strokeWidth={1.8} />
											<span>{section.label}</span>
										</Link>
									</SidebarMenuButton>
								</SidebarMenuItem>
							))}
						</SidebarMenu>
					</SidebarGroup>
				</SidebarContent>
				<SidebarFooter className="border-nav-border border-t p-2">
					<SidebarMenu>
						<SidebarMenuItem>
							<SidebarMenuButton
								asChild
								tooltip="Retour à l’application"
								className={itemClassName}
							>
								<Link to="/">
									<ArrowLeft strokeWidth={1.8} />
									<span>Retour à l’application</span>
								</Link>
							</SidebarMenuButton>
						</SidebarMenuItem>
					</SidebarMenu>
				</SidebarFooter>
			</Sidebar>
			<SidebarInset className="min-h-0 min-w-0">
				<div className="app-canvas flex min-h-0 min-w-0 flex-1 flex-col overflow-auto">
					{children}
				</div>
			</SidebarInset>
		</SidebarProvider>
	)
}
