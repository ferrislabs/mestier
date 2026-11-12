import { createFileRoute } from '@tanstack/react-router'

import { AdminStatsFeature } from '#/pages/admin-stats/feature/admin-stats-feature'

export const Route = createFileRoute('/_app/admin/')({
	component: AdminStatsFeature,
})
