import { createFileRoute, Outlet } from '@tanstack/react-router'
import { PortalLayoutFeature } from '#/pages/cursus/portal/feature/portal-layout-feature'

export const Route = createFileRoute('/_portal')({ component: PortalLayout })

/**
 * Space of the Cursus participants, who are not organization members.
 * `AuthGate` alone: `OrgGate` would send someone with no organization into
 * onboarding. `redirectState` is the full current location, so the OIDC round
 * trip comes back to the page that was asked for, invitation token included.
 */
function PortalLayout() {
	return (
		<PortalLayoutFeature>
			<Outlet />
		</PortalLayoutFeature>
	)
}
