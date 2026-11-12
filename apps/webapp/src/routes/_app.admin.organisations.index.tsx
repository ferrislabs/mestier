import { createFileRoute } from '@tanstack/react-router'

import { AdminOrganizationsFeature } from '#/pages/admin-organizations/feature/admin-organizations-feature'

export const Route = createFileRoute('/_app/admin/organisations/')({
	component: AdminOrganizationsFeature,
})
