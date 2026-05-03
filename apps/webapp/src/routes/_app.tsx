import { createFileRoute, Outlet } from '@tanstack/react-router'
import { Bell, LogOut, Search, User } from 'lucide-react'
import { useAuth } from 'react-oidc-context'

import { AppSidebar } from '#/components/app-sidebar'
import { AuthGate } from '#/components/auth-gate'
import { OrgGate } from '#/components/org-gate'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
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

					<div className="relative flex max-w-xl flex-1 items-center">
						<Search className="pointer-events-none absolute left-3 size-4 text-muted-foreground" />
						<Input
							type="search"
							placeholder="Rechercher…"
							className="pl-9 pr-16"
						/>
						<kbd className="pointer-events-none absolute right-3 hidden h-5 select-none items-center gap-0.5 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground sm:inline-flex">
							⌘K
						</kbd>
					</div>

					<div className="ml-auto flex items-center gap-2">
						<span className="hidden items-center gap-1.5 rounded-lg border bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground sm:inline-flex">
							espace:{' '}
							<span className="font-mono text-foreground">production</span>
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
