import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const NEWS_PATH = '/api/v1/news'
const NEWS_UNSEEN_PATH = '/api/v1/news/unseen'
const NEWS_SEEN_PATH = '/api/v1/news/seen'
const NEWS_ACCESS_PATH = '/api/v1/news/access'
const ADMIN_NEWS_PATH = '/api/v1/admin/news'
const ADMIN_NEWS_ITEM_PATH = '/api/v1/admin/news/{news_id}'

export type News = Schemas.NewsResponse
export type NewsRequest = Schemas.NewsRequest

const NEWS_PATHS: readonly unknown[] = [
	NEWS_PATH,
	NEWS_UNSEEN_PATH,
	ADMIN_NEWS_PATH,
]

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

export function useNewsAccess() {
	return useQuery({
		...window.tanstackApi.get(NEWS_ACCESS_PATH).queryOptions,
		staleTime: 5 * 60 * 1000,
		retry: false,
	})
}

export function useUnseenNews() {
	return useQuery({
		...window.tanstackApi.get(NEWS_UNSEEN_PATH).queryOptions,
		staleTime: Number.POSITIVE_INFINITY,
		refetchOnWindowFocus: false,
		retry: false,
	})
}

export function usePublishedNews(enabled: boolean) {
	return useQuery({
		...window.tanstackApi.get(NEWS_PATH).queryOptions,
		enabled,
	})
}

export function useMarkNewsSeen() {
	return useMutation({
		...window.tanstackApi.mutation('post', NEWS_SEEN_PATH).mutationOptions,
	})
}

export function useAdminNews(enabled: boolean) {
	return useQuery({
		...window.tanstackApi.get(ADMIN_NEWS_PATH).queryOptions,
		enabled,
	})
}

function useNewsInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => NEWS_PATHS.includes(queryId(query.queryKey)),
		})
}

export function useCreateNews() {
	const invalidate = useNewsInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', ADMIN_NEWS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useUpdateNews() {
	const invalidate = useNewsInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('put', ADMIN_NEWS_ITEM_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useDeleteNews() {
	const invalidate = useNewsInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('delete', ADMIN_NEWS_ITEM_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}
