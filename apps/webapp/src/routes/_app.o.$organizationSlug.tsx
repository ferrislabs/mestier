import {
	createFileRoute,
	Link,
	Outlet,
	useLocation,
} from '@tanstack/react-router'
import { Building2 } from 'lucide-react'

import { AppHeader } from '#/components/app-header'
import { ModuleNav } from '#/components/module-nav'
import { FullscreenMessage } from '#/components/org-gate'
import { ScopeBar } from '#/components/scope-bar'
import { Button } from '#/components/ui/button'
import { SidebarInset, SidebarProvider } from '#/components/ui/sidebar'
import {
	ActiveOrganizationProvider,
	useActiveOrganization,
	useOrganizationList,
} from '#/hooks/use-active-organization'
import { PresenceProvider } from '#/hooks/use-presence'
import { splitOrgPath } from '#/modules/org-path'
import { resolveScope } from '#/modules/scope'

export const Route = createFileRoute('/_app/o/$organizationSlug')({
	component: OrganizationLayout,
})

function OrganizationLayout() {
	const { organizationSlug } = Route.useParams()
	const organizations = useOrganizationList()
	const organization = organizations.find(
		(organization) => organization.slug === organizationSlug,
	)

	// The tenant comes from the URL: an unknown slug, or an organization the
	// user does not belong to, must say so rather than silently fall back to
	// another organization.
	if (!organization) {
		return (
			<FullscreenMessage
				icon={<Building2 className="size-8 text-muted-foreground" />}
				title="Organisation introuvable"
				message={`Aucune organisation « ${organizationSlug} » n'est accessible avec ce compte. Elle a peut-être été renommée, ou l'accès ne vous a pas été donné.`}
				action={
					<Button asChild>
						<Link to="/">Revenir à mes organisations</Link>
					</Button>
				}
			/>
		)
	}

	return (
		<ActiveOrganizationProvider activeOrganization={organization}>
			<PresenceProvider organizationId={organization.id}>
				<AppShell />
			</PresenceProvider>
		</ActiveOrganizationProvider>
	)
}

function AppShell() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const scope = resolveScope(location.pathname)
	// The homepage is its own launcher (app-launcher-ui.tsx's icon grid) — the
	// left rail would just duplicate it, so it's the one screen without one.
	const isHome = splitOrgPath(location.pathname).path === '/'

	return (
		<SidebarProvider className="flex-col">
			{/* The header spans the full width, above the sidebar rather than
			 * beside it — `ModuleNav`'s own fixed container is offset below it
			 * (see its `top-(--app-header-height)` override) so the two never
			 * occupy the same band instead of one merely painting over the
			 * other. `SidebarProvider` still wraps both: `AppHeader`'s
			 * `SidebarTrigger` needs its context regardless of layout. */}
			<AppHeader />
			<div className="flex min-h-0 flex-1">
				{isHome ? null : (
					<ModuleNav organizationSlug={activeOrganization.slug} />
				)}
				<SidebarInset>
					<ScopeBar
						label={scope.label}
						tabs={scope.tabs}
						organizationSlug={activeOrganization.slug}
					/>
					<div className="flex min-w-0 flex-1 flex-col">
						<Outlet />
					</div>
				</SidebarInset>
			</div>
		</SidebarProvider>
	)
}
