import { createFileRoute, Outlet } from '@tanstack/react-router'

import { AuthGate } from '#/components/auth-gate'
import { OrgGate } from '#/components/org-gate'

export const Route = createFileRoute('/_app')({ component: AppLayout })

/**
 * Authenticated space. It renders no application shell: that depends on the
 * tenant, resolved one level down by `/o/$organizationSlug`.
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
