import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ORG_USERS_PATH = '/api/v1/organizations/{organization_id}/users'

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isOrgUsersQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === ORG_USERS_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

export function useOrgUsers(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(ORG_USERS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useCreateOrgUser(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ORG_USERS_PATH).mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({
				predicate: (query) => isOrgUsersQuery(query.queryKey, organizationId),
			})
		},
		meta: { organizationId },
	})
}

export type OrgUser = Schemas.UserResponse
export type CreateOrgUserPayload = Schemas.CreateOrgUserRequest
