import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const INVITATIONS_PATH = '/api/v1/organizations/{organization_id}/invitations'
const INVITATION_PATH = '/api/v1/invitations/{invitation_id}'
const ACCEPT_INVITATION_PATH = '/api/v1/invitations/{token}/accept'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function invalidatePendingInvitations(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) =>
			queryKeyMeta(query.queryKey)?._id === INVITATIONS_PATH,
	})
}

/** Pending invitations for the organization — the Access column's `invited`
 * state and the "Invitations en attente" panel both read from this. */
export function usePendingInvitations(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(INVITATIONS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

/** Returns the invitation together with its clear token — see
 * `CreatedInvitation`'s doc comment: the one and only time it is readable. */
export function useCreateInvitation(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', INVITATIONS_PATH).mutationOptions,
		onSuccess: () => invalidatePendingInvitations(queryClient),
		meta: { organizationId },
	})
}

export function useRevokeInvitation() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', INVITATION_PATH).mutationOptions,
		onSuccess: () => invalidatePendingInvitations(queryClient),
	})
}

/** No organization context: the caller has none yet in the target org —
 * that is exactly what accepting grants, see `InviteAcceptFeature`. */
export function useAcceptInvitation() {
	return useMutation({
		...window.tanstackApi.mutation('post', ACCEPT_INVITATION_PATH)
			.mutationOptions,
	})
}

export type Invitation = Schemas.InvitationResponse
/** Never persisted, never refetched — the clear `token` field exists on this
 * type alone, matching the one HTTP response that ever carries it. */
export type CreatedInvitation = Schemas.CreatedInvitationResponse
