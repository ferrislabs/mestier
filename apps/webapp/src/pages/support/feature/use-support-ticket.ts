import { useSupportTicket } from '#/hooks/use-support'
import { useTicketPriorities } from '#/pages/support/feature/mocks/priorities'
import { useSupportMembers } from '#/pages/support/feature/use-support-screen'
import { toTicketVM } from '#/pages/support/lib/view-models'

const NO_DISPLAY_FIELDS: string[] = []

export function useSupportTicketVM(organizationId: string, ticketId: string) {
	const members = useSupportMembers(organizationId)
	const priorities = useTicketPriorities()
	const query = useSupportTicket(organizationId, ticketId)
	const ticket = query.data?.data
	const vm = ticket
		? toTicketVM(ticket, {
				displayFieldKeys: NO_DISPLAY_FIELDS,
				namesById: members.namesById,
				priorities: priorities.data,
			})
		: null

	return { query, vm, members }
}
