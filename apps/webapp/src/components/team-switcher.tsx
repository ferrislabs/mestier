'use client'

import { useLocation, useNavigate } from '@tanstack/react-router'
import { Building2, Check, ChevronsUpDown } from 'lucide-react'

import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	useSidebar,
} from '#/components/ui/sidebar'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { cn } from '#/lib/utils'

export function TeamSwitcher() {
	const { isMobile } = useSidebar()
	const location = useLocation()
	const navigate = useNavigate()
	const {
		organizations,
		activeOrganization,
		activeOrganizationId,
		setActiveOrganizationId,
	} = useActiveOrganization()
	const handleOrganizationSelect = (organizationId: string) => {
		if (organizationId === activeOrganizationId) return
		setActiveOrganizationId(organizationId)
		if (location.pathname.startsWith('/crm/customers/')) {
			void navigate({ to: '/crm/customers' })
		}
	}

	return (
		<SidebarMenu>
			<SidebarMenuItem>
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<SidebarMenuButton
							size="lg"
							className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
							tooltip={activeOrganization.name}
						>
							<OrganizationMark organization={activeOrganization} />
							<div className="grid flex-1 text-left text-sm leading-tight">
								<span className="truncate font-medium">
									{activeOrganization.name}
								</span>
								<span className="truncate text-xs">
									{activeOrganization.slug}
								</span>
							</div>
							<ChevronsUpDown className="ml-auto" />
						</SidebarMenuButton>
					</DropdownMenuTrigger>
					<DropdownMenuContent
						className="w-(--radix-dropdown-menu-trigger-width) min-w-72 rounded-lg"
						align="start"
						side={isMobile ? 'bottom' : 'right'}
						sideOffset={4}
					>
						<DropdownMenuLabel className="text-xs text-muted-foreground">
							Organisation active
						</DropdownMenuLabel>
						{organizations.map((organization) => {
							const selected = organization.id === activeOrganizationId
							return (
								<DropdownMenuItem
									key={organization.id}
									onClick={() => handleOrganizationSelect(organization.id)}
									className="gap-2 p-2"
								>
									<OrganizationMark
										organization={organization}
										className="size-7 rounded-md"
									/>
									<div className="grid min-w-0 flex-1 leading-tight">
										<span className="truncate font-medium">
											{organization.name}
										</span>
										<span className="truncate text-xs text-muted-foreground">
											{organization.slug}
										</span>
									</div>
									{selected ? <Check className="size-4 text-primary" /> : null}
								</DropdownMenuItem>
							)
						})}
					</DropdownMenuContent>
				</DropdownMenu>
			</SidebarMenuItem>
		</SidebarMenu>
	)
}

interface OrganizationMarkProps {
	organization: Organization
	className?: string
}

function OrganizationMark({ organization, className }: OrganizationMarkProps) {
	const initials = organization.name
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((part) => part[0]?.toUpperCase() ?? '')
		.join('')

	return (
		<div
			className={cn(
				'flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-xs font-semibold text-sidebar-primary-foreground',
				className,
			)}
		>
			{initials || <Building2 className="size-4" />}
		</div>
	)
}
