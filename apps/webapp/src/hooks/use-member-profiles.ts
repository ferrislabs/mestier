import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const PROFILE_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/profile'
const ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/archive'
const RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/restore'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateProfileReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const id = queryKeyId(query.queryKey)
			return id === PROFILES_PATH || id === PROFILE_PATH
		},
	})
}

export function useMemberProfiles(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(PROFILES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function usePutMemberProfile() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', PROFILE_PATH).mutationOptions,
		onSuccess: () => invalidateProfileReads(queryClient),
	})
}

export function useArchiveMember() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidateProfileReads(queryClient),
	})
}

export function useRestoreMember() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', RESTORE_PATH).mutationOptions,
		onSuccess: () => invalidateProfileReads(queryClient),
	})
}

export type MemberProfile = Schemas.MemberProfileResponse
export type PutMemberProfileRequest = Schemas.PutMemberProfileRequest
