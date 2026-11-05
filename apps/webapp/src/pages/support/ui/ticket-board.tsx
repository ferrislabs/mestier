import { useState } from 'react'
import { cn } from '#/lib/utils'
import type { SupportBoardUIProps, SupportStatus } from '#/pages/support/types'
import { StatusDot } from '#/pages/support/ui/status-badge'
import { KEYBOARD_HINT_ID, TicketCard } from '#/pages/support/ui/ticket-card'

export type TicketBoardProps = Pick<
	SupportBoardUIProps,
	| 'columns'
	| 'canMove'
	| 'isLoading'
	| 'draggedTicketId'
	| 'movingTicketId'
	| 'focusedTicketId'
	| 'onFocusCard'
	| 'onDragStart'
	| 'onDragEnd'
	| 'onDropOnColumn'
	| 'onShiftColumn'
	| 'onOpenCard'
>

export function TicketBoard({
	columns,
	canMove,
	isLoading,
	draggedTicketId,
	movingTicketId,
	focusedTicketId,
	onFocusCard,
	onDragStart,
	onDragEnd,
	onDropOnColumn,
	onShiftColumn,
	onOpenCard,
}: TicketBoardProps) {
	const [overStatus, setOverStatus] = useState<SupportStatus | null>(null)
	const dragged = columns
		.flatMap((column) => column.tickets)
		.find((ticket) => ticket.id === draggedTicketId)

	return (
		<div className="grid min-h-0 auto-cols-[minmax(16rem,1fr)] grid-flow-col gap-3 overflow-x-auto pb-2">
			<p id={KEYBOARD_HINT_ID} className="sr-only">
				Flèches gauche et droite pour changer de colonne.
			</p>
			{columns.map((column) => {
				const droppable =
					canMove && dragged !== undefined && dragged.status !== column.status
				return (
					<section
						key={column.status}
						aria-label={column.label}
						onDragOver={(event) => {
							if (!droppable) return
							event.preventDefault()
							event.dataTransfer.dropEffect = 'move'
							setOverStatus(column.status)
						}}
						onDragLeave={() => setOverStatus(null)}
						onDrop={(event) => {
							event.preventDefault()
							onDropOnColumn(column.status)
							setOverStatus(null)
						}}
						className={cn(
							'flex min-h-[24rem] flex-col rounded-lg border border-transparent bg-muted/50 p-2 transition-colors',
							droppable && 'border-border border-dashed',
							overStatus === column.status && 'border-ring bg-accent/60',
						)}
					>
						<header className="flex items-center gap-2 px-1.5 pt-1 pb-2.5">
							<StatusDot status={column.status} />
							<h2 className="truncate font-semibold text-sm">{column.label}</h2>
							<span className="ml-auto text-muted-foreground text-xs tabular-nums">
								{column.tickets.length}
							</span>
						</header>
						<div className="grid content-start gap-2">
							{column.tickets.map((ticket) => (
								<TicketCard
									key={ticket.id}
									ticket={ticket}
									canMove={canMove}
									dragging={ticket.id === draggedTicketId}
									moving={ticket.id === movingTicketId}
									shouldFocus={ticket.id === focusedTicketId}
									onFocus={() => onFocusCard(ticket.id)}
									onDragStart={() => onDragStart(ticket.id)}
									onDragEnd={() => {
										onDragEnd()
										setOverStatus(null)
									}}
									onShiftColumn={(direction) =>
										onShiftColumn(ticket.id, direction)
									}
									onOpen={() => onOpenCard(ticket.id)}
								/>
							))}
							{column.tickets.length === 0 && (
								<p className="px-2 py-6 text-center text-muted-foreground text-xs">
									{isLoading ? 'Chargement…' : column.description}
								</p>
							)}
						</div>
					</section>
				)
			})}
		</div>
	)
}
