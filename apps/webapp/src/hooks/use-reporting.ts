import { useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PROFITABILITY_PATH =
	'/api/v1/organizations/{organization_id}/reporting/profitability'
const WORKED_HOURS_PATH =
	'/api/v1/organizations/{organization_id}/reporting/worked-hours'

export type JobProfitability = Schemas.TaskProfitability
export type EmployeeProfitability = Schemas.EmployeeProfitability
export type WorkedHoursRow = Schemas.WorkedHoursRow

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
