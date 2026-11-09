import { createFileRoute } from '@tanstack/react-router'
import { PortalInvitationFeature } from '#/pages/cursus/portal/feature/portal-invitation-feature'

export const Route = createFileRoute('/_portal/portail/invitation/$token')({
	component: PortalInvitationRoute,
})

function PortalInvitationRoute() {
	const { token } = Route.useParams()

	return <PortalInvitationFeature token={token} />
}
