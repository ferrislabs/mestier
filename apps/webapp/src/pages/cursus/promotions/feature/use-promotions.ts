import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { invalidateCursus } from '#/pages/cursus/shared/feature/cursus-queries'
import {
	type ArchiveFilter,
	archiveQuery,
	searchQuery,
} from '#/pages/cursus/shared/lib/list-filters'

const PROMOTIONS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/promotions'
const PROMOTION_PATH =
	'/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}'
const PROMOTION_ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}/archive'

export interface PromotionFilters {
	archived: ArchiveFilter
	trainingId: string | undefined
	q?: string
}

export function usePromotions(
	organizationId: string,
	filters: PromotionFilters,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(PROMOTIONS_PATH, {
			path: { organization_id: organizationId },
			query: {
				page: 1,
				per_page: 100,
				...archiveQuery(filters.archived),
				...(filters.trainingId ? { training_id: filters.trainingId } : {}),
				...searchQuery(filters.q),
			},
		}).queryOptions,
		enabled,
	})
}

export function usePromotion(organizationId: string, promotionId: string) {
	return useQuery(
		window.tanstackApi.get(PROMOTION_PATH, {
			path: { organization_id: organizationId, promotion_id: promotionId },
		}).queryOptions,
	)
}

export function useCreatePromotion() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', PROMOTIONS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useUpdatePromotion() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', PROMOTION_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useArchivePromotion() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', PROMOTION_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export type Promotion = Schemas.PromotionResponse
