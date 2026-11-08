import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const MEMBER_DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/documents'
const MEMBER_DOCUMENT_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/documents/{document_id}'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateDocumentReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === DOCUMENTS_PATH,
	})
}

export function useMemberDocuments(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(DOCUMENTS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useAddMemberDocument() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', MEMBER_DOCUMENTS_PATH)
			.mutationOptions,
		onSuccess: () => invalidateDocumentReads(queryClient),
	})
}

export function useDeleteMemberDocument() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', MEMBER_DOCUMENT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateDocumentReads(queryClient),
	})
}

export type MemberDocumentRecord = Schemas.MemberDocumentResponse
export type AddMemberDocumentRequest = Schemas.AddMemberDocumentRequest
