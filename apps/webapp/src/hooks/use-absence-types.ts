import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const TYPES_PATH = '/api/v1/organizations/{organization_id}/absence-types'
const TYPE_PATH =
	'/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}'
const ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/archive'
const RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/restore'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateTypeReads(queryClient: ReturnType<typeof useQueryClient>) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === TYPES_PATH,
	})
}

export function useAbsenceTypes(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(TYPES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

export function useCreateAbsenceType() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', TYPES_PATH).mutationOptions,
		onSuccess: () => invalidateTypeReads(queryClient),
	})
}

export function useUpdateAbsenceType() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', TYPE_PATH).mutationOptions,
		onSuccess: () => invalidateTypeReads(queryClient),
	})
}

export function useArchiveAbsenceType() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidateTypeReads(queryClient),
	})
}

export function useRestoreAbsenceType() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', RESTORE_PATH).mutationOptions,
		onSuccess: () => invalidateTypeReads(queryClient),
	})
}

export type AbsenceType = Schemas.AbsenceTypeResponse
export type AbsenceTypeRequest = Schemas.AbsenceTypeRequest
