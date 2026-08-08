import { useLocation } from '@tanstack/react-router'
import { Bell, LogOut, User } from 'lucide-react'
import { useAuth } from 'react-oidc-context'

import { AppBreadcrumb } from '#/components/app-breadcrumb'
import { ModuleLauncher } from '#/components/module-launcher'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { SidebarTrigger } from '#/components/ui/sidebar'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'

export function AppHeader() {
	const auth = useAuth()
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const activeModule = resolveModule(splitOrgPath(location.pathname).path)
	const profile = auth.user?.profile
	const displayName =
		profile?.name ||
		profile?.preferred_username ||
		profile?.email ||
		'Utilisateur'
	const email = profile?.email ?? ''
	const initials = getInitials(displayName)

	return (
		<header className="sticky top-0 z-20 flex h-(--app-header-height) shrink-0 items-center gap-3 bg-card px-3 md:px-6">
			{/* Sur mobile le rail est un panneau : il lui faut un déclencheur. */}
			<SidebarTrigger className="-ml-1 md:hidden" />

			<AppBreadcrumb />

			<div className="ml-auto flex items-center gap-2">
				<ModuleLauncher
					activeModuleId={activeModule.id}
					organizationSlug={activeOrganization.slug}
				/>

				{/*
				 * Placeholder assumé : aucun flux d'événements n'est branché à ce jour.
				 * Le jour où il le sera, le compteur viendra du gateway temps réel —
				 * ce bouton ne doit pas être transformé en sondage périodique.
				 */}
				<Button variant="ghost" size="icon" className="text-muted-foreground">
					<Bell />
					<span className="sr-only">Notifications</span>
				</Button>

				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<button
							type="button"
							aria-label="Compte utilisateur"
							className="flex size-9 items-center justify-center rounded-full bg-primary text-sm font-medium text-primary-foreground transition-colors hover:bg-brand-muted"
						>
							{initials}
						</button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end" className="w-56">
						<DropdownMenuLabel className="font-normal">
							<div className="flex flex-col">
								<span className="truncate font-medium">{displayName}</span>
								{email ? (
									<span className="truncate text-xs text-muted-foreground">
										{email}
									</span>
								) : null}
							</div>
						</DropdownMenuLabel>
						<DropdownMenuSeparator />
						<DropdownMenuItem disabled>
							<User />
							Mon profil
						</DropdownMenuItem>
						<DropdownMenuSeparator />
						<DropdownMenuItem
							variant="destructive"
							onClick={() => {
								void auth.signoutRedirect()
							}}
						>
							<LogOut />
							Se déconnecter
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
		</header>
	)
}

function getInitials(name: string): string {
	return (
		name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('') || 'U'
	)
}
