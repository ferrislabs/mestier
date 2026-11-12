import { createFileRoute } from '@tanstack/react-router'

import { AdminLayoutFeature } from '#/pages/admin-layout/feature/admin-layout-feature'

export const Route = createFileRoute('/_app/admin')({
	component: AdminLayoutFeature,
})
