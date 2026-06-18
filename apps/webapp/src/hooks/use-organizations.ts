import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api-client'

const MY_ORGS_KEY = '/api/v1/users/@me/organizations'
const ORGANIZATION_PATH = '/api/v1/organizations/{organization_id}'

export function useMyOrganizations() {
	return useQuery(
		window.tanstackApi.get('/api/v1/users/@me/organizations').queryOptions,
	)
}

export function useCreateOrganization() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', '/api/v1/organizations')
			.mutationOptions,
		onSuccess: () => {
			void queryClient.invalidateQueries({
				queryKey: [{ _id: MY_ORGS_KEY }],
			})
		},
	})
}

export function useOrganization(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(ORGANIZATION_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useUpdateOrganization(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ORGANIZATION_PATH).mutationOptions,
		onSuccess: () => {
			void queryClient.invalidateQueries({
				predicate: (query) =>
					query.queryKey[0]?._id === MY_ORGS_KEY ||
					query.queryKey[0]?._id === ORGANIZATION_PATH,
			})
		},
		meta: { organizationId },
	})
}

export function useDeleteOrganization() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ORGANIZATION_PATH).mutationOptions,
		onSuccess: () => {
			void queryClient.invalidateQueries({
				predicate: (query) =>
					query.queryKey[0]?._id === MY_ORGS_KEY ||
					query.queryKey[0]?._id === ORGANIZATION_PATH,
			})
		},
	})
}

export type Organization = Schemas.OrganizationResponse
