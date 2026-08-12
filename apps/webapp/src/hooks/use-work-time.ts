import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const WORK_TIME_PATH = '/api/v1/members/{member_id}/work-time'
const RHYTHM_PATH = '/api/v1/members/{member_id}/rhythm'
const WORK_SLOTS_PATH = '/api/v1/members/{member_id}/work-slots'

/** An inclusive `[from, to]` window of ISO calendar days (`YYYY-MM-DD`). */
export interface WorkTimeRange {
	from: string
	to: string
}

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		member_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isWorkTimeQuery(queryKey: readonly unknown[], memberId: string) {
	const meta = queryKeyMeta(queryKey)
	return meta?._id === WORK_TIME_PATH && meta.path?.member_id === memberId
}

function invalidateWorkTime(
	queryClient: ReturnType<typeof useQueryClient>,
	memberId: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => isWorkTimeQuery(query.queryKey, memberId),
	})
}

function workTimeParams(memberId: string, range: WorkTimeRange) {
	return {
		path: { member_id: memberId },
		query: { from: range.from, to: range.to },
	}
}

/**
 * Reads the rhythm versions and dated work slots that overlap `range` for a
 * single member. `from`/`to` are required by the API (capped at 92 days).
 */
export function useWorkTime(
	memberId: string,
	range: WorkTimeRange,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(WORK_TIME_PATH, workTimeParams(memberId, range))
			.queryOptions,
		enabled: enabled && Boolean(memberId),
	})
}

/**
 * Replaces the rhythm currently in effect wholesale — see
 * `WorkTimeService::replace_rhythm` for the versioning rules this endpoint
 * enforces (same `effective_from` edits in place, a later one opens a new
 * version, an earlier one is rejected with a 409).
 */
export function useReplaceRhythm(memberId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', RHYTHM_PATH).mutationOptions,
		onSuccess: () => invalidateWorkTime(queryClient, memberId),
	})
}

/** Replaces the dated work slots within a `[from, to]` window wholesale. */
export function useReplaceWorkSlots(memberId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', WORK_SLOTS_PATH).mutationOptions,
		onSuccess: () => invalidateWorkTime(queryClient, memberId),
	})
}

export type Rhythm = Schemas.RhythmResponse
export type RhythmSlotResponse = Schemas.RhythmSlotResponse
export type WorkSlot = Schemas.WorkSlotResponse
export type WorkTime = Schemas.WorkTimeResponse
