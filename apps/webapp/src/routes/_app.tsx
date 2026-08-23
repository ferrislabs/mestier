import { createFileRoute, Outlet } from '@tanstack/react-router'

import { AuthGate } from '#/components/auth-gate'
import { GatewayStatusBanner } from '#/components/gateway-status-banner'
import { OrgGate } from '#/components/org-gate'
import { GatewayProvider } from '#/hooks/use-gateway'

export const Route = createFileRoute('/_app')({ component: AppLayout })

/**
 * Authenticated space. It renders no application shell: that depends on the
 * tenant, resolved one level down by `/o/$organizationSlug`.
 *
 * The gateway connection lives here, not in the chat module: one WebSocket
 * per session regardless of which organization or screen is active, so
 * presence and typing traffic is never multiplied by however many chat
 * screens happen to be mounted.
 */
function AppLayout() {
	return (
		<AuthGate>
			<GatewayProvider>
				<OrgGate>
					<GatewayStatusBanner />
					<Outlet />
				</OrgGate>
			</GatewayProvider>
		</AuthGate>
	)
}
