import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidateCursus } from '#/pages/cursus/participants/lib/forms'

const BASE = '/api/v1/organizations/{organization_id}/cursus'
export const ESTABLISHMENTS_PATH = `${BASE}/establishments` as const
export const ESTABLISHMENT_PATH =
	`${BASE}/establishments/{establishment_id}` as const
export const ESTABLISHMENT_ARCHIVE_PATH =
	`${BASE}/establishments/{establishment_id}/archive` as const
export const TRAININGS_PATH = `${BASE}/trainings` as const
export const TRAINING_PATH = `${BASE}/trainings/{training_id}` as const
export const TRAINING_ARCHIVE_PATH =
	`${BASE}/trainings/{training_id}/archive` as const
export const CLOSURE_REASONS_PATH = `${BASE}/closure-reasons` as const
export const CLOSURE_REASON_PATH =
	`${BASE}/closure-reasons/{closure_reason_id}` as const
export const CLOSURE_REASON_ARCHIVE_PATH =
	`${BASE}/closure-reasons/{closure_reason_id}/archive` as const

export function useEstablishments(
	organizationId: string,
	archived = false,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(ESTABLISHMENTS_PATH, {
			path: { organization_id: organizationId },
			query: archived ? { archived: true } : {},
		}).queryOptions,
		enabled,
	})
}

export function useTrainings(
	organizationId: string,
	archived = false,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TRAININGS_PATH, {
			path: { organization_id: organizationId },
			query: archived ? { archived: true, per_page: 100 } : { per_page: 100 },
		}).queryOptions,
		enabled,
	})
}

export function useClosureReasons(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(CLOSURE_REASONS_PATH, {
			path: { organization_id: organizationId },
			query: {},
		}).queryOptions,
	)
}

export function useCreateEstablishment() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', ESTABLISHMENTS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'establishments'),
	})
}

export function useUpdateEstablishment() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', ESTABLISHMENT_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'establishments'),
	})
}

export function useArchiveEstablishment() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', ESTABLISHMENT_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'establishments'),
	})
}

export function useCreateTraining() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', TRAININGS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'trainings'),
	})
}

export function useUpdateTraining() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', TRAINING_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'trainings'),
	})
}

export function useArchiveTraining() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', TRAINING_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'trainings'),
	})
}

export function useCreateClosureReason() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', CLOSURE_REASONS_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'closure-reasons'),
	})
}

export function useUpdateClosureReason() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', CLOSURE_REASON_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'closure-reasons'),
	})
}

export function useArchiveClosureReason() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', CLOSURE_REASON_ARCHIVE_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'closure-reasons'),
	})
}
