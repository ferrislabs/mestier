import { useRouterState } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { useAuth } from 'react-oidc-context'
import { AuthGate } from '#/components/auth-gate'
import { PortalHeader } from '#/pages/cursus/portal/ui/portal-header'

function PortalShell({ children }: { children: ReactNode }) {
	const auth = useAuth()
	const profile = auth.user?.profile
	const displayName =
		profile?.name ||
		profile?.preferred_username ||
		profile?.email ||
		'Utilisateur'

	return (
		<div className="min-h-screen bg-background">
			<PortalHeader
				displayName={displayName}
				onSignOut={() => void auth.signoutRedirect()}
			/>
			<main>{children}</main>
		</div>
	)
}

export function PortalLayoutFeature({ children }: { children: ReactNode }) {
	const href = useRouterState({ select: (state) => state.location.href })

	return (
		<AuthGate redirectState={href}>
			<PortalShell>{children}</PortalShell>
		</AuthGate>
	)
}
