import { createFileRoute } from '@tanstack/react-router'
import { AuthGate } from '#/components/auth-gate'
import { InviteAcceptFeature } from '#/pages/invite/feature/invite-accept-feature'

export const Route = createFileRoute('/invite/$token')({
	component: InviteAcceptPage,
})

/**
 * Deliberately outside `/_app`: `OrgGate` would redirect a fresh invitee —
 * who has zero organizations by construction — straight into onboarding
 * before they ever reach this screen (see issue #185's "layout problem").
 * `AuthGate` alone still authenticates; `redirectState` is what lets the
 * OIDC round trip come back here instead of the app's fixed `redirect_uri`.
 */
function InviteAcceptPage() {
	const { token } = Route.useParams()

	return (
		<AuthGate redirectState={`/invite/${token}`}>
			<InviteAcceptFeature token={token} />
		</AuthGate>
	)
}
