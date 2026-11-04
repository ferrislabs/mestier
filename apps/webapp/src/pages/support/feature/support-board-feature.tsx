import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useMoveSupportTicket } from '#/hooks/use-support'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	SupportOrganizationUnavailable,
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
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import { toCardVM } from '#/pages/support/lib/view-models'
import type {
	SupportColumnVM,
	SupportScreenProps,
	SupportStatus,
} from '#/pages/support/types'
import { SupportBoardUI } from '#/pages/support/ui/support-board-ui'
import { SupportToolbar } from '#/pages/support/ui/support-toolbar'

export function SupportBoardFeature({
	filters,
	onFiltersChange,
	onOpenTicket,
}: SupportScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportBoardScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			filters={filters}
			onFiltersChange={onFiltersChange}
			onOpenTicket={onOpenTicket}
		/>
	)
}

interface SupportBoardScreenProps extends SupportScreenProps {
	organizationId: string
	organizationName: string
}

function SupportBoardScreen({
	organizationId,
	organizationName,
	filters,
	onFiltersChange,
	onOpenTicket,
}: SupportBoardScreenProps) {
	const canMove = useHasPermission('MANAGE_SUPPORT')
	const data = useSupportScreenData(organizationId, filters)
	const { tickets, fieldKeys, namesById } = data
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
		cards: grouped[column.status].map((ticket) =>
			toCardVM(ticket, fieldKeys, namesById),
		),
	}))

	const toolbar = (
		<SupportToolbar
			{...data.toolbarProps}
			filters={filters}
			onChange={(patch) => onFiltersChange({ ...filters, ...patch })}
			onClear={() => onFiltersChange(EMPTY_SUPPORT_FILTERS)}
		/>
	)

	return (
		<SupportBoardUI
			organizationName={organizationName}
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
