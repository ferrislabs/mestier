import { createFileRoute } from '@tanstack/react-router'
import { PortalTriadFeature } from '#/pages/cursus/portal/feature/portal-triad-feature'

export const Route = createFileRoute(
	'/_portal/portail/$organizationSlug/trinomes/$triadId',
)({
	component: PortalTriadRoute,
})

function PortalTriadRoute() {
	const { organizationSlug, triadId } = Route.useParams()

	return (
		<PortalTriadFeature organizationSlug={organizationSlug} triadId={triadId} />
	)
}
