import {
	useMutation,
	useQueries,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidateMaintenance } from '#/hooks/maintenance-cache'

const INTERVENTIONS_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions'
const VALIDATE_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate'
const REJECT_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject'
const GRID_PATH = '/api/v1/organizations/{organization_id}/maintenance/grid'

/** What the manager's queue lists: the response carries ids, not labels. */
export type MaintenanceIntervention = {
	created_at: string
	done_on: string
	id: Schemas.InterventionId
	line_id: Schemas.MaintenanceLineId
	note?: (string | null) | undefined
	proposed_by: Schemas.MemberId
	status: Schemas.InterventionStatus
	task_id?: (null | Schemas.TaskId) | undefined
}

export type MaintenanceGrid = Schemas.Grid

/** The proposed visits, oldest first: the queue a manager works through. */
export function useProposedInterventions(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(INTERVENTIONS_PATH, {
			path: { organization_id: organizationId },
			query: { status: 'proposed', page: 1, per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/**
 * The grid of each season asked for. It is the only response that names a
 * line and its site, and a visit only carries the id of its line.
 */
export function useMaintenanceGrids(organizationId: string, years: number[]) {
	return useQueries({
		queries: years.map((season) => ({
			...window.tanstackApi.get(GRID_PATH, {
				path: { organization_id: organizationId },
				query: { season },
			}).queryOptions,
			enabled: Boolean(organizationId),
		})),
	})
}

export function useValidateIntervention() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', VALIDATE_PATH).mutationOptions,
		onSettled: () => invalidateMaintenance(queryClient),
	})
}

export function useRejectIntervention() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', REJECT_PATH).mutationOptions,
		onSettled: () => invalidateMaintenance(queryClient),
	})
}
