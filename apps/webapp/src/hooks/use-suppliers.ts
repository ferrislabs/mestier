import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const SUPPLIERS_PATH = '/api/v1/organizations/{organization_id}/suppliers'
const SUPPLIER_PATH =
	'/api/v1/organizations/{organization_id}/suppliers/{supplier_id}'
const SUPPLIER_ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/archive'
const SUPPLIER_RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/restore'

export type Supplier = Schemas.SupplierResponse
export type SupplierPayload = Schemas.CreateSupplierRequest

export function useSuppliers(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(SUPPLIERS_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useSupplier(
	organizationId: string,
	supplierId: string,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(SUPPLIER_PATH, {
			path: { organization_id: organizationId, supplier_id: supplierId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId) && Boolean(supplierId),
	})
}

function useSupplierInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => {
				const meta = query.queryKey[0]
				const id =
					typeof meta === 'object' && meta !== null
						? (meta as { _id?: unknown })._id
						: undefined
				return id === SUPPLIERS_PATH || id === SUPPLIER_PATH
			},
		})
}

export function useCreateSupplier() {
	const invalidate = useSupplierInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', SUPPLIERS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useUpdateSupplier() {
	const invalidate = useSupplierInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('put', SUPPLIER_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useArchiveSupplier() {
	const invalidate = useSupplierInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', SUPPLIER_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useRestoreSupplier() {
	const invalidate = useSupplierInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', SUPPLIER_RESTORE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}
