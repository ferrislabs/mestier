import { useLocation, useNavigate } from '@tanstack/react-router'
import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { type CSSProperties, useState } from 'react'

import { CreateOrganizationDialog } from '#/components/create-organization-dialog'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
	CommandSeparator,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { cn } from '#/lib/utils'
import { organizationHue } from '#/modules/org-hue'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { crossOrganizationPath } from '#/modules/scope'

/**
 * Organization switcher, rendered at the root of the breadcrumb: the
 * organization is the first level of scope, so that is where it is changed.
 */
export function OrgSwitcher() {
	const location = useLocation()
	const navigate = useNavigate()
	const { organizations, activeOrganization, activeOrganizationId } =
		useActiveOrganization()
	const [open, setOpen] = useState(false)
	const [isCreateOpen, setCreateOpen] = useState(false)

	const handleOrganizationSelect = (organization: Organization) => {
		setOpen(false)
		if (organization.id === activeOrganizationId) return

		const { path } = splitOrgPath(location.pathname)

		void navigate({
			to: buildOrgPath(organization.slug, crossOrganizationPath(path)),
		})
	}

	return (
		<>
			<Popover open={open} onOpenChange={setOpen}>
				<PopoverTrigger asChild>
					<button
						type="button"
						aria-label={`Organisation active : ${activeOrganization.name}. Changer d’organisation`}
						className="flex h-7 min-w-0 items-center gap-1.5 rounded-md border bg-card px-1.5 font-medium text-[13px] outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
					>
						<OrganizationBadge
							organization={activeOrganization}
							className="size-4 text-[9px]"
						/>
						<span className="truncate">{activeOrganization.name}</span>
						<ChevronsUpDown
							className="size-3.5 shrink-0 text-muted-foreground"
							aria-hidden
						/>
					</button>
				</PopoverTrigger>
				<PopoverContent align="start" sideOffset={6} className="w-64 p-0">
					<Command>
						<CommandInput placeholder="Rechercher une organisation…" />
						<CommandList>
							<CommandEmpty>Aucune organisation.</CommandEmpty>
							<CommandGroup>
								{organizations.map((organization) => (
									<CommandItem
										key={organization.id}
										value={`${organization.name} ${organization.slug}`}
										onSelect={() => handleOrganizationSelect(organization)}
									>
										<OrganizationBadge organization={organization} />
										<span className="truncate">{organization.name}</span>
										{organization.id === activeOrganizationId ? (
											<Check className="ml-auto text-highlight" />
										) : null}
									</CommandItem>
								))}
							</CommandGroup>
							<CommandSeparator />
							<CommandGroup>
								<CommandItem
									value="Créer une organisation"
									onSelect={() => {
										setOpen(false)
										setCreateOpen(true)
									}}
								>
									<Plus />
									Créer une organisation
								</CommandItem>
							</CommandGroup>
						</CommandList>
					</Command>
				</PopoverContent>
			</Popover>

			{isCreateOpen ? (
				<CreateOrganizationDialog open onOpenChange={setCreateOpen} />
			) : null}
		</>
	)
}

interface OrganizationBadgeProps {
	organization: Pick<Organization, 'name' | 'slug'>
	className?: string
}

function OrganizationBadge({
	organization,
	className,
}: OrganizationBadgeProps) {
	return (
		<span
			className={cn(
				'flex size-5 shrink-0 items-center justify-center rounded-[5px] bg-[oklch(0.55_0.12_var(--hue))] font-semibold text-[10px] text-white',
				className,
			)}
			style={{ '--hue': organizationHue(organization.slug) } as CSSProperties}
			aria-hidden
		>
			{organization.name.charAt(0).toUpperCase()}
		</span>
	)
}
