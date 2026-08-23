import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidatePlanning } from '#/hooks/use-planning'

const ASSIGNMENT_REPORTS_PATH =
	'/api/v1/organizations/{organization_id}/assignment-reports'
const ASSIGNMENT_REPORT_RESOLUTION_PATH =
	'/api/v1/assignment-reports/{assignment_report_id}/resolution'

export type AssignmentReport = Schemas.AssignmentReportResponse
export type AssignmentReportResolution = Schemas.AssignmentReportResolution
export type PaginationMetadata = Schemas.PaginationMetadata

interface QueryKeyMeta {
	_id?: unknown
	path?: { organization_id?: unknown }
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isAssignmentReportsListQuery(
	queryKey: readonly unknown[],
	organizationId: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === ASSIGNMENT_REPORTS_PATH &&
		meta.path?.organization_id === organizationId
	)
}

/**
 * The organization's reports, one `resolution` at a time — the backend list
 * endpoint has no "every state" mode (see `libs/handlers-planning/src/
 * assignment_report/list.rs`: an absent filter defaults to `PENDING`), so the
 * three resolutions are three distinct requests rather than one unfiltered
 * one narrowed client-side.
 */
export function useAssignmentReports(
	organizationId: string,
	resolution: AssignmentReportResolution,
	page: number,
	perPage: number,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(ASSIGNMENT_REPORTS_PATH, {
			path: { organization_id: organizationId },
			query: { resolution, page, per_page: perPage },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

/**
 * The pending count alone — cheap (`per_page: 1`, only `pagination.total`
 * matters) — for the badge the calendar and team views show without opening
 * anything.
 */
export function usePendingAssignmentReportsCount(organizationId: string) {
	const query = useAssignmentReports(organizationId, 'PENDING', 1, 1)
	return query.data?.pagination?.total ?? null
}

function invalidateAssignmentReports(
	queryClient: ReturnType<typeof useQueryClient>,
	organizationId: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			isAssignmentReportsListQuery(query.queryKey, organizationId),
	})
}

/**
 * The manager's arbitration — `PATCH .../resolution`. Never touches the
 * task itself (see `AssignmentReportService::resolve_report`'s own doc):
 * applying a report is recording a decision, moving the plan is the
 * existing task `PATCH`, and the feature layer chains the two in that order.
 */
export function useResolveAssignmentReport(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ASSIGNMENT_REPORT_RESOLUTION_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await Promise.all([
				invalidateAssignmentReports(queryClient, organizationId),
				// Applying moves the task's own duration via a separate `PATCH`
				// the caller issues first; invalidating the grid here too means
				// a manager who resolves from the reports list still sees it
				// reflected without a manual refresh.
				invalidatePlanning(queryClient),
			])
		},
	})
}
