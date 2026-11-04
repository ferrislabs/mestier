import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { applyStatus } from '#/pages/support/lib/columns'
import { supportFiltersToQuery } from '#/pages/support/lib/filters'
import type { SupportFilters, SupportStatus } from '#/pages/support/types'

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const TICKET_STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'

export const SUPPORT_TICKETS_PER_PAGE = 100

function supportTicketsRequest(
	organizationId: string,
	filters: SupportFilters,
) {
	return {
		path: { organization_id: organizationId },
		query: supportFiltersToQuery(filters, SUPPORT_TICKETS_PER_PAGE),
	}
}

export function useSupportTickets(
	organizationId: string,
	filters: SupportFilters,
) {
	return useQuery({
		...window.tanstackApi.get(
			TICKETS_PATH,
			supportTicketsRequest(organizationId, filters),
		).queryOptions,
		enabled: Boolean(organizationId),
	})
}

interface SupportTicketsPage {
	data: Schemas.TicketResponse[]
	pagination?: Schemas.PaginationMetadata | null
}

interface MoveVariables {
	path: { organization_id: string; ticket_id: string }
	body: { status: SupportStatus }
}

export function useMoveSupportTicket(
	organizationId: string,
	filters: SupportFilters,
) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(
		TICKETS_PATH,
		supportTicketsRequest(organizationId, filters),
	).queryKey

	return useMutation({
		...window.tanstackApi.mutation('patch', TICKET_STATUS_PATH).mutationOptions,
		onMutate: async (variables) => {
			const { path, body } = variables as MoveVariables
			await queryClient.cancelQueries({ queryKey })

			const previous = queryClient.getQueryData<SupportTicketsPage>(queryKey)
			if (previous) {
				queryClient.setQueryData<SupportTicketsPage>(queryKey, {
					...previous,
					data: applyStatus(previous.data, path.ticket_id, body.status),
				})
			}

			return { previous }
		},
		onError: (_error, _variables, context) => {
			const previous = (
				context as { previous?: SupportTicketsPage } | undefined
			)?.previous
			if (previous) queryClient.setQueryData(queryKey, previous)
		},
		onSettled: () => queryClient.invalidateQueries({ queryKey }),
	})
}

export function useSupportSettings(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}
