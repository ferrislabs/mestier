import { useMutation, useQueryClient } from '@tanstack/react-query'

const BADGES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges'
const BADGE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}'
const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'

function isTicketQuery(queryKey: readonly unknown[], organizationId: string) {
	const meta = queryKey[0]
	if (typeof meta !== 'object' || meta === null) return false
	const { _id, path } = meta as {
		_id?: unknown
		path?: { organization_id?: unknown }
	}

	return (
		typeof _id === 'string' &&
		_id.startsWith(TICKETS_PATH) &&
		path?.organization_id === organizationId
	)
}

function useRefreshTickets(organizationId: string) {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => isTicketQuery(query.queryKey, organizationId),
		})
}

export function useAddSupportTicketBadge(organizationId: string) {
	return useMutation({
		...window.tanstackApi.mutation('post', BADGES_PATH).mutationOptions,
		onSettled: useRefreshTickets(organizationId),
	})
}

export function useRemoveSupportTicketBadge(organizationId: string) {
	return useMutation({
		...window.tanstackApi.mutation('delete', BADGE_PATH).mutationOptions,
		onSettled: useRefreshTickets(organizationId),
	})
}
