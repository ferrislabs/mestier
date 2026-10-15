import {
	useMutation,
	useQueries,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const MAINTENANCE_PREFIX = '/api/v1/organizations/{organization_id}/maintenance'
const CONTRACTS_PATH = `${MAINTENANCE_PREFIX}/contracts`
const CONTRACT_PATH = `${MAINTENANCE_PREFIX}/contracts/{contract_id}`
const ARCHIVE_PATH = `${MAINTENANCE_PREFIX}/contracts/{contract_id}/archive`
const DUPLICATE_PATH = `${MAINTENANCE_PREFIX}/contracts/{contract_id}/duplicate`
const PROJECTS_PATH = '/api/v1/organizations/{organization_id}/projects'
const CUSTOMER_CONTEXTS_PATH =
	'/api/v1/customers/{customer_id}/customer-contexts'

export type MaintenanceContract = Schemas.MaintenanceContractResponse

function queryId(queryKey: readonly unknown[]): unknown {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

/**
 * Every maintenance read: contracts, the grid, and whatever the recap and
 * validation screens read from the same prefix. A contract change moves all of
 * them, and so does a recorded visit.
 */
export function isMaintenanceQuery(queryKey: readonly unknown[]): boolean {
	const id = queryId(queryKey)
	return typeof id === 'string' && id.startsWith(MAINTENANCE_PREFIX)
}

export function useInvalidateMaintenance() {
	const queryClient = useQueryClient()

	return () =>
		Promise.all([
			queryClient.invalidateQueries({
				predicate: (query) => isMaintenanceQuery(query.queryKey),
			}),
			// A contract owns a project, created with it.
			queryClient.invalidateQueries({
				predicate: (query) => queryId(query.queryKey) === PROJECTS_PATH,
			}),
		])
}

export function useMaintenanceContracts(
	organizationId: string,
	season: number,
	includeArchived: boolean,
) {
	return useQuery({
		...window.tanstackApi.get(CONTRACTS_PATH, {
			path: { organization_id: organizationId },
			query: { season, include_archived: includeArchived },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCreateMaintenanceContract() {
	const invalidate = useInvalidateMaintenance()

	return useMutation({
		...window.tanstackApi.mutation('post', CONTRACTS_PATH).mutationOptions,
		onSuccess: () => invalidate(),
	})
}

export function useUpdateMaintenanceContract() {
	const invalidate = useInvalidateMaintenance()

	return useMutation({
		...window.tanstackApi.mutation('put', CONTRACT_PATH).mutationOptions,
		onSuccess: () => invalidate(),
	})
}

export function useArchiveMaintenanceContract() {
	const invalidate = useInvalidateMaintenance()

	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidate(),
	})
}

export function useDuplicateMaintenanceContract() {
	const invalidate = useInvalidateMaintenance()

	return useMutation({
		...window.tanstackApi.mutation('post', DUPLICATE_PATH).mutationOptions,
		onSuccess: () => invalidate(),
	})
}

/**
 * The sites of several customers at once, for a list that names the site of
 * every contract. Same request, so same cache entry, as `useCustomerContexts`.
 */
export function useSitesOfCustomers(customerIds: string[]) {
	return useQueries({
		queries: customerIds.map((customerId) => ({
			...window.tanstackApi.get(CUSTOMER_CONTEXTS_PATH, {
				path: { customer_id: customerId },
				query: { page: 1, per_page: 100 },
			}).queryOptions,
			enabled: Boolean(customerId),
		})),
	})
}
