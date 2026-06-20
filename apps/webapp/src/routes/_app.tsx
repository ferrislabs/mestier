import { createFileRoute, Outlet } from '@tanstack/react-router'
import { Bell, LogOut, User } from 'lucide-react'
import { useAuth } from 'react-oidc-context'

import { AppBreadcrumb } from '#/components/app-breadcrumb'
import { AppSidebar } from '#/components/app-sidebar'
import { AuthGate } from '#/components/auth-gate'
import { OrgGate } from '#/components/org-gate'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	SidebarInset,
	SidebarProvider,
	SidebarTrigger,
} from '#/components/ui/sidebar'
import { useActiveOrganization } from '#/hooks/use-active-organization'

export const Route = createFileRoute('/_app')({ component: AppLayout })

function AppLayout() {
	return (
		<AuthGate>
			<OrgGate>
				<AppShell />
			</OrgGate>
		</AuthGate>
	)
}

function AppShell() {
	const auth = useAuth()
	const { activeOrganization } = useActiveOrganization()
	const profile = auth.user?.profile
	const displayName =
		profile?.name ||
		profile?.preferred_username ||
		profile?.email ||
		'Utilisateur'
	const email = profile?.email ?? ''
	const initials = getInitials(displayName)

	return (
		<SidebarProvider>
			<AppSidebar />
			<SidebarInset>
				<header className="sticky top-0 z-10 flex h-16 shrink-0 items-center gap-3 border-b bg-card/90 px-3 backdrop-blur md:px-6">
					<SidebarTrigger className="-ml-1" />

					<AppBreadcrumb />

					<div className="ml-auto flex items-center gap-2">
						<span className="hidden items-center gap-1.5 rounded-lg border bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground sm:inline-flex">
							org:{' '}
							<span className="max-w-40 truncate font-medium text-foreground">
								{activeOrganization.name}
							</span>
						</span>
						<Button
							variant="ghost"
							size="icon"
							className="rounded-lg text-muted-foreground"
						>
							<Bell />
							<span className="sr-only">Notifications</span>
						</Button>

						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<button
									type="button"
									className="flex size-9 items-center justify-center rounded-lg bg-primary text-sm font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-brand-muted"
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
				<div className="flex flex-1 flex-col">
					<Outlet />
				</div>
			</SidebarInset>
		</SidebarProvider>
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
