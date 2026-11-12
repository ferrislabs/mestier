import { useAdminStats } from '#/hooks/use-admin'
import { mutationErrorMessage } from '#/lib/api-error'
import { AdminStats } from '#/pages/admin-stats/ui/admin-stats'

export function AdminStatsFeature() {
	const stats = useAdminStats()

	return (
		<AdminStats
			stats={stats.data?.data}
			loading={stats.isPending}
			error={mutationErrorMessage(stats.error)}
		/>
	)
}
