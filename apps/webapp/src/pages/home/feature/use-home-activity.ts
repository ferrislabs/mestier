import { useQuery } from '@tanstack/react-query'
import { MOCK_HOME_ACTIVITY } from '#/pages/home/feature/mocks/activity'

const LATENCY_MS = 150

export function useHomeActivity(organizationId: string) {
	return useQuery({
		queryKey: ['home', 'activity', organizationId],
		queryFn: async () => {
			await new Promise((resolve) => setTimeout(resolve, LATENCY_MS))
			return MOCK_HOME_ACTIVITY
		},
		staleTime: Number.POSITIVE_INFINITY,
	})
}
