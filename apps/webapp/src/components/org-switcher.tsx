import { useLocation, useNavigate } from '@tanstack/react-router'
import { Building2, Check, ChevronsUpDown } from 'lucide-react'

import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { cn } from '#/lib/utils'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { crossOrganizationPath } from '#/modules/scope'

/**
 * Sélecteur d'organisation, rendu à la racine du fil d'Ariane : l'organisation
 * est le premier niveau de scope, c'est donc là qu'on en change.
 */
export function OrgSwitcher() {
	const location = useLocation()
	const navigate = useNavigate()
	const { organizations, activeOrganization, activeOrganizationId } =
		useActiveOrganization()

	// Changer d'organisation est une navigation : le tenant vit dans l'URL. On
	// reste sur le même écran, sauf s'il désigne une entité de l'organisation
	// quittée — un identifiant de client n'existe pas dans l'organisation cible.
	const handleOrganizationSelect = (organization: Organization) => {
		if (organization.id === activeOrganizationId) return

		const { path } = splitOrgPath(location.pathname)

		void navigate({
			to: buildOrgPath(organization.slug, crossOrganizationPath(path)),
		})
	}

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<button
					type="button"
					aria-label="Changer d'organisation"
					className="flex min-w-0 items-center gap-1.5 rounded-lg px-1.5 py-1 text-sm font-medium text-foreground transition-colors hover:bg-muted"
				>
					<span className="max-w-40 truncate">{activeOrganization.name}</span>
					<ChevronsUpDown className="size-3.5 shrink-0 text-muted-foreground" />
				</button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="start" className="min-w-72 rounded-lg">
				<DropdownMenuLabel className="text-xs text-muted-foreground">
					Organisation active
				</DropdownMenuLabel>
				{organizations.map((organization) => {
					const selected = organization.id === activeOrganizationId
					return (
						<DropdownMenuItem
							key={organization.id}
							onClick={() => handleOrganizationSelect(organization)}
							className="gap-2 p-2"
						>
							<OrganizationMark organization={organization} />
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
				'flex aspect-square size-7 items-center justify-center rounded-md bg-primary text-xs font-semibold text-primary-foreground',
				className,
			)}
		>
			{initials || <Building2 className="size-4" />}
		</div>
	)
}
