import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ORGANIZATION_CONTEXTS_PATH =
	'/api/v1/organizations/{organization_id}/organization-contexts'
const ORGANIZATION_CONTEXT_PATH = '/api/v1/organization-contexts/{context_id}'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function invalidateOrganizationContextList(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			queryKeyMeta(query.queryKey)?._id === ORGANIZATION_CONTEXTS_PATH,
	})
}

export function useOrganizationContexts(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(ORGANIZATION_CONTEXTS_PATH, {
			path: { organization_id: organizationId },
			query: { page: 1, per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCreateOrganizationContext(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ORGANIZATION_CONTEXTS_PATH)
			.mutationOptions,
		onSuccess: () => invalidateOrganizationContextList(queryClient),
		meta: { organizationId },
	})
}

export function useUpdateOrganizationContext(_organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ORGANIZATION_CONTEXT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateOrganizationContextList(queryClient),
	})
}

export function useDeleteOrganizationContext(_organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ORGANIZATION_CONTEXT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateOrganizationContextList(queryClient),
	})
}

export type OrganizationContext = Schemas.OrganizationContextResponse
export type CreateOrganizationContextPayload =
	Schemas.CreateOrganizationContextRequest
export type UpdateOrganizationContextPayload =
	Schemas.UpdateOrganizationContextRequest
