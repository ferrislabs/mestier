import { createFileRoute } from '@tanstack/react-router'
import { PortalSpaceFeature } from '#/pages/cursus/portal/feature/portal-space-feature'

export const Route = createFileRoute('/_portal/portail/$organizationSlug/')({
	component: PortalSpaceRoute,
})

function PortalSpaceRoute() {
	const { organizationSlug } = Route.useParams()

	return <PortalSpaceFeature organizationSlug={organizationSlug} />
}
