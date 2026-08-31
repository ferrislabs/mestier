import { Link, useLocation } from '@tanstack/react-router'
import { Bell, LogOut, User } from 'lucide-react'
import { useAuth } from 'react-oidc-context'

import { AppBreadcrumb } from '#/components/app-breadcrumb'
import { MestierAppIcon } from '#/components/brand/mestier-logo'
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
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'

export function AppHeader() {
	const auth = useAuth()
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const path = splitOrgPath(location.pathname).path
	const activeModule = resolveModule(path)
	// No left rail on the homepage (see `_app.o.$organizationSlug.tsx`) — its
	// mobile trigger would just open nothing.
	const isHome = path === '/'
	const profile = auth.user?.profile
	const displayName =
		profile?.name ||
		profile?.preferred_username ||
		profile?.email ||
		'Utilisateur'
	const email = profile?.email ?? ''
	const initials = getInitials(displayName)

	return (
		<header className="sticky top-0 z-20 flex h-(--app-header-height) shrink-0 items-center gap-2 border-b bg-card px-3 md:px-6">
			{/* The brand mark, not `ModuleNav`'s job anymore — each module is its
			 * own app with its own navigation (see `module-nav.tsx`'s own doc);
			 * identity belongs at the root of the header, the way a Google app
			 * shows its mark next to the product name rather than in a side rail. */}
			<Link
				to={buildOrgPath(activeOrganization.slug, '/')}
				aria-label="Accueil"
				className="flex shrink-0 items-center"
			>
				<MestierAppIcon className="size-8 shrink-0 rounded-none" />
			</Link>

			{isHome ? null : (
				// Sur mobile le rail est un panneau : il lui faut un déclencheur.
				<SidebarTrigger className="border border-foreground/20 md:hidden" />
			)}

			<AppBreadcrumb />

			<div className="ml-auto flex items-center gap-2">
				<ModuleLauncher
					activeModuleId={activeModule.id}
					organizationSlug={activeOrganization.slug}
				/>

				{/*
				 * A deliberate placeholder: no event stream is wired up yet. The day
				 * one is, the counter will come from the realtime gateway — this
				 * button must not be turned into periodic polling.
				 */}
				<Button
					variant="ghost"
					size="icon"
					className="border border-foreground/20 text-muted-foreground"
				>
					<Bell />
					<span className="sr-only">Notifications</span>
				</Button>

				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<button
							type="button"
							aria-label="Compte utilisateur"
							className="flex size-9 items-center justify-center rounded-none bg-primary text-sm font-medium text-primary-foreground transition-colors hover:bg-brand-muted"
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
