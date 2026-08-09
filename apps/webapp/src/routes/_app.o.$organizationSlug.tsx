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
			<AppShell />
		</ActiveOrganizationProvider>
	)
}

function AppShell() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const scope = resolveScope(location.pathname)

	return (
		<SidebarProvider>
			<ModuleNav organizationSlug={activeOrganization.slug} />
			<SidebarInset>
				<AppHeader />
				<ScopeBar
					label={scope.label}
					tabs={scope.tabs}
					organizationSlug={activeOrganization.slug}
				/>
				<div className="flex min-w-0 flex-1 flex-col">
					<Outlet />
				</div>
			</SidebarInset>
		</SidebarProvider>
	)
}
