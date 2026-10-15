import { useMutation, useQuery } from '@tanstack/react-query'
import { useInvalidateMaintenance } from '#/hooks/use-maintenance-contracts'

const GRID_PATH = '/api/v1/organizations/{organization_id}/maintenance/grid'
const RECORD_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions'

/** The season sheet. The due dates in it are computed by the backend, as of today. */
export function useMaintenanceGrid(organizationId: string, season: number) {
	return useQuery({
		...window.tanstackApi.get(GRID_PATH, {
			path: { organization_id: organizationId },
			query: { season },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/** A manager's direct entry: validated at once, so the due date moves. */
export function useRecordMaintenanceIntervention() {
	const invalidate = useInvalidateMaintenance()

	return useMutation({
		...window.tanstackApi.mutation('post', RECORD_PATH).mutationOptions,
		onSuccess: () => invalidate(),
	})
}
