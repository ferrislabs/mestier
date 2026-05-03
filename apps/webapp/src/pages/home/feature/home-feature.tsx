import { HomeUI } from '#/pages/home/ui/home-ui'

export function HomeFeature() {
	const stats = {
		customers: 3,
		inventory: 0,
		invoices: 0,
		revenueMonth: 0,
	}

	return <HomeUI userName="Nathael" stats={stats} />
}
