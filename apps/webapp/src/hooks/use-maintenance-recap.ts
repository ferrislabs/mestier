import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import {
	invalidateMaintenance,
	invalidateTasks,
} from '#/hooks/maintenance-cache'

const RECAP_PATH = '/api/v1/organizations/{organization_id}/maintenance/recap'
const PLAN_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/recap/plan'

export type MaintenanceRecap = Schemas.Recap
export type MaintenanceRecapGroup = Schemas.RecapGroup
export type MaintenanceRecapLine = Schemas.RecapLine

/** `week` is an ISO week, `2026-W43`. */
export function useMaintenanceRecap(organizationId: string, week: string) {
	return useQuery({
		...window.tanstackApi.get(RECAP_PATH, {
			path: { organization_id: organizationId },
			query: { week },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(week),
	})
}

export function maintenanceRecapPdfPath(
	organizationId: string,
	week: string,
): string {
	return `/api/v1/organizations/${organizationId}/maintenance/recap.pdf?week=${encodeURIComponent(week)}`
}

/**
 * Turns recap lines into backlog tasks, all or nothing. Refreshes the recap
 * (the lines leave it) and the task lists (a task each appears on the board).
 */
export function usePlanMaintenanceLines() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PLAN_PATH).mutationOptions,
		onSettled: async () => {
			await Promise.all([
				invalidateMaintenance(queryClient),
				invalidateTasks(queryClient),
			])
		},
	})
}
