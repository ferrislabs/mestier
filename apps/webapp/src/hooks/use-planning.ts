import { useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'

/** An inclusive `[from, to]` window of ISO calendar days (`YYYY-MM-DD`), capped at 92 days by the API. */
export interface PlanningRange {
	from: string
	to: string
}

/**
 * Reads the team planning — resources, entries and work time — for `range`.
 * `from`/`to` are required by the API; see `computeWindow` for how a
 * `view`/`date` pair in the URL turns into this range.
 */
export function usePlanning(
	organizationId: string,
	range: PlanningRange,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(PLANNING_PATH, {
			path: { organization_id: organizationId },
			query: { from: range.from, to: range.to },
		}).queryOptions,
		enabled:
			enabled &&
			Boolean(organizationId) &&
			Boolean(range.from) &&
			Boolean(range.to),
	})
}

export type PlanningResource = Schemas.PlanningResourceResponse
export type PlanningEntry = Schemas.PlanningEntryResponse
export type PlanningWorkTime = Schemas.PlanningWorkTimeResponse
export type PlanningResponse = Schemas.PlanningResponse
