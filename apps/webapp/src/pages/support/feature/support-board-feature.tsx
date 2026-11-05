import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useMoveSupportTicket } from '#/hooks/use-support'
import { mutationErrorMessage } from '#/lib/api-error'
import { SupportTicketDetailFeature } from '#/pages/support/feature/support-ticket-detail-feature'
import {
	resetFilters,
	SupportOrganizationUnavailable,
	screenTitles,
	useSupportScreenData,
} from '#/pages/support/feature/use-support-screen'
import {
	computeColumnShift,
	computeDrop,
	groupTicketsByStatus,
	SUPPORT_COLUMNS,
	type SupportMove,
	supportStatusLabel,
} from '#/pages/support/lib/columns'
import type {
	SupportColumnVM,
	SupportScreenProps,
	SupportStatus,
} from '#/pages/support/types'
import { SupportBoardUI } from '#/pages/support/ui/support-board-ui'
import { TicketModalUI } from '#/pages/support/ui/ticket-modal-ui'
import { TicketsToolbar } from '#/pages/support/ui/tickets-toolbar'

export function SupportBoardFeature(props: SupportScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportBoardScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

interface SupportBoardScreenProps extends SupportScreenProps {
	organizationId: string
}

function SupportBoardScreen({
	organizationId,
	filters,
	onFiltersChange,
	onOpenTicket,
	onOpenFullPage,
	onViewChange,
}: SupportBoardScreenProps) {
	const canMove = useHasPermission('MANAGE_SUPPORT')
	const data = useSupportScreenData(organizationId, filters, 'board')
	const { tickets } = data
	const moveTicket = useMoveSupportTicket(organizationId, filters)
	const [draggedTicketId, setDraggedTicketId] = useState<string | null>(null)
	const [focusedTicketId, setFocusedTicketId] = useState<string | null>(null)
	const [announcement, setAnnouncement] = useState('')

	function applyMove(ticketId: string, move: SupportMove) {
		if (!move.changed) return
		moveTicket.mutate({
			path: { organization_id: organizationId, ticket_id: ticketId },
			body: { status: move.status },
		})
	}

	function handleDropOnColumn(status: SupportStatus) {
		const ticketId = draggedTicketId
		setDraggedTicketId(null)
		if (!ticketId || !canMove) return

		applyMove(ticketId, computeDrop(tickets, ticketId, status))
	}

	function handleShiftColumn(ticketId: string, direction: 1 | -1) {
		if (!canMove) return

		const move = computeColumnShift(tickets, ticketId, direction)
		if (!move.changed) return

		const subject =
			tickets.find((ticket) => ticket.id === ticketId)?.subject ?? 'Le ticket'
		setFocusedTicketId(ticketId)
		setAnnouncement(
			`${subject} déplacé vers ${supportStatusLabel(move.status)}`,
		)
		applyMove(ticketId, move)
	}

	const grouped = groupTicketsByStatus(tickets)
	const columns: SupportColumnVM[] = SUPPORT_COLUMNS.map((column) => ({
		status: column.status,
		label: column.label,
		description: column.description,
		tickets: grouped[column.status],
	}))

	const titles = screenTitles(filters, data.openCount)

	const toolbar = (
		<TicketsToolbar
			{...data.toolbarProps}
			title={titles.title}
			count={tickets.length}
			filters={filters}
			showStatus={false}
			view="board"
			onViewChange={onViewChange}
			onChange={(patch) => onFiltersChange({ ...filters, ...patch })}
			onReset={() => onFiltersChange(resetFilters(filters))}
		/>
	)

	return (
		<SupportBoardUI
			title={titles.title}
			description={titles.description}
			columns={columns}
			canMove={canMove}
			isLoading={data.isLoading}
			error={
				data.ticketsQuery.error?.message ??
				mutationErrorMessage(moveTicket.error) ??
				null
			}
			onRetry={() => void data.ticketsQuery.refetch()}
			announcement={announcement}
			toolbar={toolbar}
			modal={
				filters.ticket ? (
					<TicketModalUI onClose={() => onOpenTicket(null)}>
						<SupportTicketDetailFeature
							key={filters.ticket}
							organizationId={organizationId}
							ticketId={filters.ticket}
							variant="modal"
							siblings={data.allTickets}
							onOpenTicket={onOpenTicket}
							onOpenFullPage={onOpenFullPage}
							onClose={() => onOpenTicket(null)}
						/>
					</TicketModalUI>
				) : null
			}
			emptyReason={data.emptyReason}
			truncationNotice={data.truncationNotice}
			draggedTicketId={draggedTicketId}
			movingTicketId={
				moveTicket.isPending
					? ((
							moveTicket.variables as
								| { path?: { ticket_id?: string } }
								| undefined
						)?.path?.ticket_id ?? null)
					: null
			}
			focusedTicketId={focusedTicketId}
			onFocusCard={setFocusedTicketId}
			onDragStart={setDraggedTicketId}
			onDragEnd={() => setDraggedTicketId(null)}
			onDropOnColumn={handleDropOnColumn}
			onShiftColumn={handleShiftColumn}
			onOpenCard={onOpenTicket}
		/>
	)
}
