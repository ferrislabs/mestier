import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const LEGAL_MENTION_TEMPLATES_PATH =
	'/api/v1/organizations/{organization_id}/legal-mention-templates'
const LEGAL_MENTION_TEMPLATE_PATH =
	'/api/v1/legal-mention-templates/{template_id}'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function invalidateLegalMentionList(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			queryKeyMeta(query.queryKey)?._id === LEGAL_MENTION_TEMPLATES_PATH,
	})
}

export function useLegalMentionTemplates(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(LEGAL_MENTION_TEMPLATES_PATH, {
			path: { organization_id: organizationId },
			query: { page: 1, per_page: 100 },
		}).queryOptions,
	)
}

export function useCreateLegalMentionTemplate(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', LEGAL_MENTION_TEMPLATES_PATH)
			.mutationOptions,
		onSuccess: () => invalidateLegalMentionList(queryClient),
		meta: { organizationId },
	})
}

export function useUpdateLegalMentionTemplate(_organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', LEGAL_MENTION_TEMPLATE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateLegalMentionList(queryClient),
	})
}

export function useDeleteLegalMentionTemplate(_organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', LEGAL_MENTION_TEMPLATE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateLegalMentionList(queryClient),
	})
}

export type LegalMentionTemplate = Schemas.LegalMentionTemplateResponse
export type CreateLegalMentionTemplatePayload =
	Schemas.CreateLegalMentionTemplateRequest
export type UpdateLegalMentionTemplatePayload =
	Schemas.UpdateLegalMentionTemplateRequest
