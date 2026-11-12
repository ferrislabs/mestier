import { createFileRoute } from '@tanstack/react-router'

import { AdminUsersFeature } from '#/pages/admin-users/feature/admin-users-feature'

export const Route = createFileRoute('/_app/admin/utilisateurs')({
	component: AdminUsersFeature,
})
