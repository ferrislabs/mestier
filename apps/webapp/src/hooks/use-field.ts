import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/field/tasks'
const CURRENT_PATH = '/api/v1/organizations/{organization_id}/field/current'
const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/field/time-entries'
const STOP_PATH = '/api/v1/field/time-entries/{time_entry_id}/stop'
const PHOTOS_PATH = '/api/v1/field/time-entries/{time_entry_id}/photos'
const DAY_END_PATH = '/api/v1/organizations/{organization_id}/field/day-end'

export type FieldTask = Schemas.FieldTaskResponse
export type TimeEntry = Schemas.TimeEntryResponse
export type TimeEntryPhoto = Schemas.TimeEntryPhotoResponse
export type PhotoPhase = Schemas.TimeEntryPhotoPhase
export type DayLog = Schemas.DayLogResponse

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

export function useAttachFieldPhoto(organizationId: string) {
	const invalidate = useInvalidateField(organizationId)

	return useMutation({
		...window.tanstackApi.mutation('post', PHOTOS_PATH).mutationOptions,
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
