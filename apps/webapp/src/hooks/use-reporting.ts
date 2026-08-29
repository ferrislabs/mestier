import { useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PROFITABILITY_PATH =
	'/api/v1/organizations/{organization_id}/reporting/profitability'
const WORKED_HOURS_PATH =
	'/api/v1/organizations/{organization_id}/reporting/worked-hours'

/** #306: money fields are `null` when the caller lacks `VIEW_COST` — see
 * `ProfitabilityResponse::costs_redacted` on the response itself. */
export type ProjectProfitability = Schemas.ProjectProfitabilityResponse
export type MemberProfitability = Schemas.MemberProfitabilityResponse
export type WorkedHoursRow = Schemas.WorkedHoursRow
export type MissingCost = Schemas.MissingCost

export interface Period {
	/** Both ends included, as the API reads them. */
	from: string
	to: string
}

/**
 * Profitability over a period.
 *
 * The period is sent as two dates and turned into instants server-side, in the
 * organization's timezone. The browser deliberately does not do that
 * conversion: a laptop set to another zone would shift a month's boundary.
 *
 * Every figure comes from the plan, not from a clock. Nobody points any more,
 * so a task that was scheduled and assigned is a cost whether or not somebody
 * registered anything.
 */
export function useProfitability(organizationId: string, period: Period) {
	return useQuery({
		...window.tanstackApi.get(PROFITABILITY_PATH, {
			path: { organization_id: organizationId },
			query: { from: period.from, to: period.to },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useWorkedHours(organizationId: string, period: Period) {
	return useQuery({
		...window.tanstackApi.get(WORKED_HOURS_PATH, {
			path: { organization_id: organizationId },
			query: { from: period.from, to: period.to },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}
