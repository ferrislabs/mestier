import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidateCursus } from '#/pages/cursus/shared/feature/cursus-queries'
import type { ArchiveFilter } from '#/pages/cursus/shared/lib/list-filters'

const SCHOOL_TUTORS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/school-tutors'
const SCHOOL_TUTOR_PATH =
	'/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}'
const ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}/archive'
const RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}/restore'

export const SCHOOL_TUTORS_PER_PAGE = 25

export interface SchoolTutorsQuery {
	archived: ArchiveFilter
	q?: string
	page?: number
	per_page?: number
}

export function useSchoolTutors(
	organizationId: string,
	query: SchoolTutorsQuery,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(SCHOOL_TUTORS_PATH, {
			path: { organization_id: organizationId },
			query,
		}).queryOptions,
		enabled,
	})
}

export function useCreateSchoolTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', SCHOOL_TUTORS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useUpdateSchoolTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', SCHOOL_TUTOR_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useArchiveSchoolTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useRestoreSchoolTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', RESTORE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}
