import { useActiveOrganization } from '#/hooks/use-active-organization'
import { SupportTicketDetailFeature } from '#/pages/support/feature/support-ticket-detail-feature'
import {
	resetFilters,
	SupportOrganizationUnavailable,
	screenTitles,
	useSupportScreenData,
} from '#/pages/support/feature/use-support-screen'
import type { SupportScreenProps } from '#/pages/support/types'
import { SupportListUI } from '#/pages/support/ui/support-list-ui'
import { TicketListHeader } from '#/pages/support/ui/ticket-list-header'

export function SupportListFeature(props: SupportScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportListScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

interface SupportListScreenProps extends SupportScreenProps {
	organizationId: string
}

function SupportListScreen({
	organizationId,
	filters,
	onFiltersChange,
	onOpenTicket,
	onOpenFullPage,
	onViewChange,
}: SupportListScreenProps) {
	const data = useSupportScreenData(organizationId, filters, 'list')
	const titles = screenTitles(filters, data.openCount)

	const header = (
		<TicketListHeader
			{...data.toolbarProps}
			title={titles.title}
			count={data.tickets.length}
			filters={filters}
			showStatus
			view="list"
			onViewChange={(view) => {
				onOpenTicket(null)
				onViewChange(view)
			}}
			onChange={(patch) => onFiltersChange({ ...filters, ...patch })}
			onReset={() => onFiltersChange(resetFilters(filters))}
		/>
	)

	return (
		<SupportListUI
			header={header}
			tickets={data.tickets}
			selectedTicketId={filters.ticket ?? null}
			isLoading={data.isLoading}
			error={data.ticketsQuery.error?.message ?? null}
			onRetry={() => void data.ticketsQuery.refetch()}
			emptyReason={data.emptyReason}
			truncationNotice={data.truncationNotice}
			onOpenRow={onOpenTicket}
			detail={
				filters.ticket ? (
					<SupportTicketDetailFeature
						key={filters.ticket}
						organizationId={organizationId}
						ticketId={filters.ticket}
						variant="pane"
						siblings={data.allTickets}
						onOpenTicket={onOpenTicket}
						onOpenFullPage={onOpenFullPage}
						onClose={() => onOpenTicket(null)}
					/>
				) : null
			}
		/>
	)
}
