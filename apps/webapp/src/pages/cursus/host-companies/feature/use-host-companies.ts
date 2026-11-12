import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidateCursus } from '#/pages/cursus/participants/lib/forms'

export const HOST_COMPANIES_PATH =
	'/api/v1/organizations/{organization_id}/cursus/host-companies'
export const HOST_COMPANY_PATH =
	'/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}'
export const HOST_COMPANY_ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}/archive'

export function useHostCompanies(
	organizationId: string,
	query: { archived?: boolean; page?: number; per_page?: number } = {},
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(HOST_COMPANIES_PATH, {
			path: { organization_id: organizationId },
			query,
		}).queryOptions,
		enabled,
	})
}

export function useHostCompany(organizationId: string, hostCompanyId: string) {
	return useQuery(
		window.tanstackApi.get(HOST_COMPANY_PATH, {
			path: { organization_id: organizationId, host_company_id: hostCompanyId },
		}).queryOptions,
	)
}

export function useCreateHostCompany() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', HOST_COMPANIES_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'host-companies'),
	})
}

export function useUpdateHostCompany() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', HOST_COMPANY_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'host-companies'),
	})
}

export function useArchiveHostCompany() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', HOST_COMPANY_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'host-companies'),
	})
}
