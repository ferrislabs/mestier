import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'

import {
	SidebarGroup,
	SidebarGroupLabel,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
} from '#/components/ui/sidebar'

export interface NavItem {
	title: string
	to: string
	icon?: LucideIcon
	exact?: boolean
	disabled?: boolean
	badge?: string | number
}

interface NavMainProps {
	label?: string
	items: NavItem[]
}

export function NavMain({ label, items }: NavMainProps) {
	return (
		<SidebarGroup>
			{label ? (
				<SidebarGroupLabel className="text-[10px] font-semibold uppercase tracking-wider text-white/80">
					{label}
				</SidebarGroupLabel>
			) : null}
			<SidebarMenu>
				{items.map((item) => (
					<SidebarMenuItem key={item.title}>
						{item.disabled ? (
							<SidebarMenuButton
								tooltip={`${item.title} · bientôt`}
								className="cursor-not-allowed rounded-lg font-medium text-white/55 hover:bg-transparent hover:text-white/55"
							>
								{item.icon ? <item.icon /> : null}
								<span>{item.title}</span>
								<span className="ml-auto rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 text-[10px] font-medium text-white/55 group-data-[collapsible=icon]:hidden">
									soon
								</span>
							</SidebarMenuButton>
						) : (
							<SidebarMenuButton asChild tooltip={item.title}>
								<Link
									to={item.to}
									activeOptions={item.exact ? { exact: true } : undefined}
									activeProps={{ 'data-active': 'true' }}
									className="rounded-lg font-medium text-white transition-colors hover:bg-white/10 hover:text-white data-[active=true]:bg-white/15 data-[active=true]:font-semibold data-[active=true]:text-white data-[active=true]:[&_.nav-badge]:bg-white/20 data-[active=true]:[&_.nav-badge]:text-white data-[active=true]:[&_svg]:text-white [&_svg]:text-white"
								>
									{item.icon ? <item.icon /> : null}
									<span>{item.title}</span>
									{item.badge !== undefined ? (
										<span className="nav-badge ml-auto rounded-md bg-white/10 px-1.5 py-0.5 text-xs font-semibold text-white/85 group-data-[collapsible=icon]:hidden">
											{item.badge}
										</span>
									) : null}
								</Link>
							</SidebarMenuButton>
						)}
					</SidebarMenuItem>
				))}
			</SidebarMenu>
		</SidebarGroup>
	)
}
