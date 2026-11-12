import { createFileRoute } from '@tanstack/react-router'

import { AdminOrganizationFeature } from '#/pages/admin-organizations/feature/admin-organization-feature'

export const Route = createFileRoute(
	'/_app/admin/organisations/$organizationId',
)({
	component: AdminOrganizationRoute,
})

function AdminOrganizationRoute() {
	const { organizationId } = Route.useParams()
	return <AdminOrganizationFeature organizationId={organizationId} />
}
