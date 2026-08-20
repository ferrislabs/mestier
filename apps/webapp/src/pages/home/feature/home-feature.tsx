import { useActiveOrganization } from '#/hooks/use-active-organization'
import { MyTasksTodayFeature } from '#/pages/home/feature/my-tasks-today-feature'
import { HomeUI } from '#/pages/home/ui/home-ui'

export function HomeFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const stats = {
		customers: 3,
		inventory: 0,
		invoices: 0,
		revenueMonth: 0,
	}

	return (
		<HomeUI
			userName="Nathael"
			stats={stats}
			todayTasks={<MyTasksTodayFeature organizationId={activeOrganizationId} />}
		/>
	)
}
