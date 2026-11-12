import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invalidateCursus } from '#/pages/cursus/participants/lib/forms'

export const PARTICIPANTS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants'
export const PARTICIPANT_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}'
export const APPRENTICE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/apprentice'
export const COMPANY_TUTORS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/company-tutors'
export const COMPANY_TUTOR_PATH =
	'/api/v1/organizations/{organization_id}/cursus/company-tutors/{company_tutor_id}'
export const INVITATION_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/invitation'
export const DEACTIVATE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/deactivate'
export const REACTIVATE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/reactivate'

export const PARTICIPANTS_PER_PAGE = 25

export interface ParticipantsQuery {
	kind?: 'apprentice' | 'company_tutor'
	status?: 'pending' | 'active' | 'inactive'
	host_company_id?: string
	page?: number
	per_page?: number
}

export function useParticipants(
	organizationId: string,
	query: ParticipantsQuery,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(PARTICIPANTS_PATH, {
			path: { organization_id: organizationId },
			query,
		}).queryOptions,
		enabled,
	})
}

export function useParticipant(organizationId: string, participantId: string) {
	return useQuery(
		window.tanstackApi.get(PARTICIPANT_PATH, {
			path: { organization_id: organizationId, participant_id: participantId },
		}).queryOptions,
	)
}

export function useCreateParticipant() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', PARTICIPANTS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useUpdateParticipant() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', PARTICIPANT_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useUpsertApprentice() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('put', APPRENTICE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useAddCompanyTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', COMPANY_TUTORS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useRemoveCompanyTutor() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('delete', COMPANY_TUTOR_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useInviteParticipant() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', INVITATION_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useDeactivateParticipant() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', DEACTIVATE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}

export function useReactivateParticipant() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', REACTIVATE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient, 'participants'),
	})
}
