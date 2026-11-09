import { useMutation, useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const SPACES_PATH = '/api/v1/cursus/me/spaces'
const MY_TRIADS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/me/triads'
const MY_TRIAD_PATH =
	'/api/v1/organizations/{organization_id}/cursus/me/triads/{triad_id}'
const ACCEPT_PATH = '/api/v1/cursus/invitations/{token}/accept'

export type CursusSpace = Schemas.CursusSpaceResponse
export type MyTriad = Schemas.MyTriadResponse
export type MyTriadDetail = Schemas.MyTriadDetailResponse

const SPACES_STALE_TIME = 5 * 60 * 1000

export function useMyCursusSpaces(enabled = true) {
	return useQuery({
		...window.tanstackApi.get(SPACES_PATH).queryOptions,
		staleTime: SPACES_STALE_TIME,
		enabled,
	})
}

export function useMyCursusTriads(organizationId: string | undefined) {
	return useQuery({
		...window.tanstackApi.get(MY_TRIADS_PATH, {
			path: { organization_id: organizationId ?? '' },
		}).queryOptions,
		enabled: organizationId !== undefined,
	})
}

export function useMyCursusTriad(
	organizationId: string | undefined,
	triadId: string,
) {
	return useQuery({
		...window.tanstackApi.get(MY_TRIAD_PATH, {
			path: { organization_id: organizationId ?? '', triad_id: triadId },
		}).queryOptions,
		enabled: organizationId !== undefined,
		retry: false,
	})
}

export function useAcceptCursusInvitation() {
	return useMutation(
		window.tanstackApi.mutation('post', ACCEPT_PATH).mutationOptions,
	)
}
