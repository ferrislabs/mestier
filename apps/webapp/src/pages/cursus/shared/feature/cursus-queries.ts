import { type QueryClient, useQueries, useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'

const TRAININGS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/trainings'
const CLOSURE_REASONS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/closure-reasons'
const HOST_COMPANIES_PATH =
	'/api/v1/organizations/{organization_id}/cursus/host-companies'
const PARTICIPANTS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants'
const PARTICIPANT_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}'

const PAGE = { page: 1, per_page: 100 }

export function invalidateCursus(queryClient: QueryClient) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const meta = query.queryKey[0]
			if (typeof meta !== 'object' || meta === null) return false
			const id = (meta as { _id?: unknown })._id
			return typeof id === 'string' && id.includes('/cursus/')
		},
	})
}

export function useTrainings(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(TRAININGS_PATH, {
			path: { organization_id: organizationId },
			query: PAGE,
		}).queryOptions,
		enabled,
	})
}

export function useClosureReasons(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(CLOSURE_REASONS_PATH, {
			path: { organization_id: organizationId },
			query: {},
		}).queryOptions,
		enabled,
	})
}

export function useHostCompanies(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(HOST_COMPANIES_PATH, {
			path: { organization_id: organizationId },
			query: PAGE,
		}).queryOptions,
		enabled,
	})
}

export function useOrganizationMembers(organizationId: string, enabled = true) {
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
		members: enabled,
	})
	return catalog.members.data?.data ?? []
}

export function useParticipantDetails(
	organizationId: string,
	kind: 'apprentice' | 'company_tutor',
	enabled: boolean,
): { isLoading: boolean; details: Schemas.ParticipantDetailResponse[] } {
	const list = useQuery({
		...window.tanstackApi.get(PARTICIPANTS_PATH, {
			path: { organization_id: organizationId },
			query: { kind, ...PAGE },
		}).queryOptions,
		enabled,
	})
	const rows = list.data?.data ?? []

	const details = useQueries({
		queries: rows.map((row) => ({
			...window.tanstackApi.get(PARTICIPANT_PATH, {
				path: { organization_id: organizationId, participant_id: row.id },
			}).queryOptions,
			enabled,
		})),
		combine: (results) => ({
			isLoading: results.some((result) => result.isLoading),
			data: results.flatMap((result) =>
				result.data?.data ? [result.data.data] : [],
			),
		}),
	})

	return {
		isLoading: list.isLoading || details.isLoading,
		details: details.data,
	}
}
