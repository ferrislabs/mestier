'use client'

import { Building2, Check, ChevronsUpDown, Plus } from 'lucide-react'

import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
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
	const {
		organizations,
		activeOrganization,
		activeOrganizationId,
		setActiveOrganizationId,
	} = useActiveOrganization()

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
									onClick={() => setActiveOrganizationId(organization.id)}
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
						<DropdownMenuSeparator />
						<DropdownMenuItem disabled className="gap-2 p-2">
							<div className="flex size-7 items-center justify-center rounded-md border bg-transparent">
								<Plus className="size-4" />
							</div>
							<div className="grid leading-tight">
								<span className="font-medium">Ajouter une organisation</span>
								<span className="text-xs text-muted-foreground">
									Bientôt disponible
								</span>
							</div>
						</DropdownMenuItem>
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
