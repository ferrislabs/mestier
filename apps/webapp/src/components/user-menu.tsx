import { Link } from '@tanstack/react-router'
import { GraduationCap, LogOut, Settings } from 'lucide-react'
import { useAuth } from 'react-oidc-context'

import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { useHasPortalSpace } from '#/pages/cursus/me/feature/use-cursus-participation'

interface UserMenuProps {
	className?: string
	side?: 'bottom' | 'right' | 'top'
	/** Show the name and email next to the initials, as in the side nav. */
	showIdentity?: boolean
}

/** Account menu of the signed-in user: profile and sign-out. */
export function UserMenu({
	className,
	side = 'bottom',
	showIdentity = false,
}: UserMenuProps) {
	const auth = useAuth()
	const { activeOrganization } = useActiveOrganization()
	const hasPortalSpace = useHasPortalSpace()
	const profile = auth.user?.profile
	const displayName =
		profile?.name ||
		profile?.preferred_username ||
		profile?.email ||
		'Utilisateur'
	const email = profile?.email ?? ''

	return (
		<DropdownMenu>
			<DropdownMenuTrigger
				aria-label="Menu du compte"
				className={cn(
					'flex items-center gap-2.5 rounded-md text-left outline-none focus-visible:ring-2 focus-visible:ring-ring',
					showIdentity &&
						'w-full px-2 py-1.5 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0',
					className,
				)}
			>
				<span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-nav-primary font-semibold text-[11px] text-nav-primary-foreground">
					{getInitials(displayName)}
				</span>
				{showIdentity ? (
					<span className="min-w-0 leading-tight group-data-[collapsible=icon]:hidden">
						<span className="block truncate font-medium text-[13px]">
							{displayName}
						</span>
						{email ? (
							<span className="block truncate text-[11px] text-nav-muted">
								{email}
							</span>
						) : null}
					</span>
				) : null}
			</DropdownMenuTrigger>
			<DropdownMenuContent side={side} align="end" className="w-60">
				<DropdownMenuLabel className="font-normal">
					<span className="block truncate font-medium">{displayName}</span>
					{email ? (
						<span className="block truncate text-muted-foreground text-xs">
							{email}
						</span>
					) : null}
				</DropdownMenuLabel>
				<DropdownMenuSeparator />
				<DropdownMenuItem asChild>
					<Link to={buildOrgPath(activeOrganization.slug, '/settings')}>
						<Settings />
						Paramètres
					</Link>
				</DropdownMenuItem>
				{hasPortalSpace ? (
					<DropdownMenuItem asChild>
						<Link to="/portail">
							<GraduationCap />
							Mon portail Cursus
						</Link>
					</DropdownMenuItem>
				) : null}
				<DropdownMenuItem
					onClick={() => {
						void auth.signoutRedirect()
					}}
				>
					<LogOut />
					Se déconnecter
				</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

function getInitials(name: string): string {
	return (
		name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((word) => word[0]?.toUpperCase() ?? '')
			.join('') || 'U'
	)
}
