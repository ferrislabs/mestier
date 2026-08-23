import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/field/tasks'
const CURRENT_PATH = '/api/v1/organizations/{organization_id}/field/current'
const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/field/time-entries'
const DECLARE_PATH =
	'/api/v1/organizations/{organization_id}/field/time-entries/declare'
const STOP_PATH = '/api/v1/field/time-entries/{time_entry_id}/stop'
const PHOTOS_PATH = '/api/v1/field/time-entries/{time_entry_id}/photos'
const RECOVER_PATH = '/api/v1/field/time-entries/{time_entry_id}/recover'
const DAY_END_PATH = '/api/v1/organizations/{organization_id}/field/day-end'
const REPORT_ASSIGNMENT_PATH =
	'/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report'
const ASSIGNMENT_REPORT_PATH =
	'/api/v1/field/assignment-reports/{assignment_report_id}'
const ASSIGNMENT_REPORTS_PATH =
	'/api/v1/organizations/{organization_id}/field/assignment-reports'

export type FieldTask = Schemas.FieldTaskResponse
export type TimeEntry = Schemas.TimeEntryResponse
export type TimeEntryPhoto = Schemas.TimeEntryPhotoResponse
export type PhotoPhase = Schemas.TimeEntryPhotoPhase
export type DayLog = Schemas.DayLogResponse
export type AssignmentReport = Schemas.AssignmentReportResponse
export type AssignmentReportResolution = Schemas.AssignmentReportResolution

/**
 * The caller's jobs for a day.
 *
 * `work_date` is left out on purpose: the server resolves "today" in the
 * organization's timezone, which the browser cannot be trusted to match. A
 * phone set to another zone would otherwise ask for the wrong day.
 */
export function useMyFieldTasks(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(TASKS_PATH, {
			path: { organization_id: organizationId },
			query: {},
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/**
 * What the caller is clocked on to, or null.
 *
 * Refetched when the tab regains focus: a worker who locked their phone during
 * a job comes back to a screen that must not offer to start another one.
 */
export function useCurrentTimeEntry(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(CURRENT_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
		refetchOnWindowFocus: true,
	})
}

/** Everything the field screen shows derives from these two, so both refresh together. */
function useInvalidateField(organizationId: string) {
	const queryClient = useQueryClient()

	return () =>
		Promise.all([
			queryClient.invalidateQueries({
				queryKey: window.tanstackApi.get(CURRENT_PATH, {
					path: { organization_id: organizationId },
				}).queryKey,
			}),
			queryClient.invalidateQueries({
				queryKey: window.tanstackApi.get(TASKS_PATH, {
					path: { organization_id: organizationId },
					query: {},
				}).queryKey,
			}),
		])
}

export function useStartTimeEntry(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', ENTRIES_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useStopTimeEntry(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', STOP_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/**
 * Declares a stretch of work that was never clocked live at all — the rush
 * that left no time to press "start". Distinct from `useRecoverTimeEntry`,
 * which only closes an entry that is already open; this one has nothing
 * open to begin with, so both ends are given at once.
 */
export function useDeclareTimeEntry(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', DECLARE_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useAttachFieldPhoto(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', PHOTOS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/**
 * Closes a stretch the worker forgot, at the time they now declare.
 *
 * Separate from `useStopTimeEntry` because the API keeps them apart: stopping
 * records the moment it happens, recovering records a moment being recalled,
 * and the second is marked so no report reads it as the first.
 */
export function useRecoverTimeEntry(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', RECOVER_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useEndWorkingDay(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', DAY_END_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

interface AssignmentReportsQueryKeyMeta {
	_id?: unknown
	path?: { organization_id?: unknown }
}

function isAssignmentReportsListQuery(
	queryKey: readonly unknown[],
	organizationId: string,
) {
	const meta = queryKey[0]
	if (typeof meta !== 'object' || meta === null) return false
	const { _id, path } = meta as AssignmentReportsQueryKeyMeta
	return (
		_id === ASSIGNMENT_REPORTS_PATH &&
		(path as { organization_id?: unknown } | undefined)?.organization_id ===
			organizationId
	)
}

/**
 * The caller's own reports — resolved ones included, so a worker can see
 * that their word was acted on. One page is enough for a day's worth of
 * jobs, so this deliberately does not paginate the way `useTaskComments`
 * does for a whole thread.
 */
export function useMyAssignmentReports(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(ASSIGNMENT_REPORTS_PATH, {
			path: { organization_id: organizationId },
			query: { page: 1, per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
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

export function useReportAssignment(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', REPORT_ASSIGNMENT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateAssignmentReports(queryClient, organizationId),
	})
}

/** `PATCH` — the backend refuses once a manager has resolved the report (see
 * `AssignmentReportService::amend_report`); the UI only offers this while
 * the report it targets is still pending. */
export function useAmendAssignmentReport(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ASSIGNMENT_REPORT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateAssignmentReports(queryClient, organizationId),
	})
}

export function useWithdrawAssignmentReport(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ASSIGNMENT_REPORT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateAssignmentReports(queryClient, organizationId),
	})
}
