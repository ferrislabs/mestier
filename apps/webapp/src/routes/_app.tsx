import { createFileRoute, Outlet } from '@tanstack/react-router'

import { AuthGate } from '#/components/auth-gate'
import { OrgGate } from '#/components/org-gate'

export const Route = createFileRoute('/_app')({ component: AppLayout })

/**
 * Espace authentifié. Il ne rend aucune coque applicative : celle-ci dépend du
 * tenant, résolu un cran plus bas par `/o/$organizationSlug`.
 */
function AppLayout() {
	return (
		<AuthGate>
			<OrgGate>
				<Outlet />
			</OrgGate>
		</AuthGate>
	)
}
