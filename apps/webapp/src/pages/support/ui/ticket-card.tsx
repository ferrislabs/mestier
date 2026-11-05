import { Loader2 } from 'lucide-react'
import { type KeyboardEvent, useEffect, useRef } from 'react'
import { cn } from '#/lib/utils'
import { formatRelative } from '#/pages/support/lib/format'
import type { SupportTicketVM } from '#/pages/support/types'
import { AssigneeStack } from '#/pages/support/ui/assignee-stack'
import {
	BadgeChip,
	CustomFieldChip,
} from '#/pages/support/ui/custom-field-chip'
import { PriorityBadge } from '#/pages/support/ui/priority-badge'

export const KEYBOARD_HINT_ID = 'support-card-keyboard-hint'

export function TicketCard({
	ticket,
	canMove,
	dragging,
	moving,
	shouldFocus,
	onFocus,
	onDragStart,
	onDragEnd,
	onShiftColumn,
	onOpen,
}: {
	ticket: SupportTicketVM
	canMove: boolean
	dragging: boolean
	moving: boolean
	shouldFocus: boolean
	onFocus: () => void
	onDragStart: () => void
	onDragEnd: () => void
	onShiftColumn: (direction: 1 | -1) => void
	onOpen: () => void
}) {
	const reference = useRef<HTMLButtonElement>(null)

	useEffect(() => {
		if (shouldFocus) reference.current?.focus()
	}, [shouldFocus])

	function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
		if (!canMove) return
		if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return

		if (event.key === 'ArrowLeft') {
			event.preventDefault()
			onShiftColumn(-1)
		} else if (event.key === 'ArrowRight') {
			event.preventDefault()
			onShiftColumn(1)
		}
	}

	return (
		<button
			type="button"
			ref={reference}
			draggable={canMove ? true : undefined}
			aria-describedby={canMove ? KEYBOARD_HINT_ID : undefined}
			onClick={onOpen}
			onFocus={onFocus}
			onKeyDown={handleKeyDown}
			onDragStart={(event) => {
				if (!canMove) return
				event.dataTransfer.setData('text/plain', ticket.id)
				event.dataTransfer.effectAllowed = 'move'
				onDragStart()
			}}
			onDragEnd={canMove ? onDragEnd : undefined}
			className={cn(
				'grid w-full gap-2 rounded-md border border-border bg-card p-3 text-left shadow-xs outline-none transition-[box-shadow,opacity] hover:border-ring/60 hover:shadow-sm focus-visible:ring-2 focus-visible:ring-ring',
				dragging && 'opacity-40',
			)}
		>
			<span className="flex items-start justify-between gap-2">
				<span className="font-mono text-muted-foreground text-xs">
					{ticket.numberLabel}
				</span>
				<span className="flex flex-wrap justify-end gap-1">
					{moving && (
						<span>
							<Loader2
								aria-hidden
								className="size-3.5 animate-spin text-muted-foreground"
							/>
							<span className="sr-only">Déplacement en cours</span>
						</span>
					)}
					{ticket.badges.map((badge) => (
						<BadgeChip
							key={badge.label}
							label={badge.label}
							tone={badge.tone}
						/>
					))}
				</span>
			</span>
			<span className="line-clamp-2 block font-medium text-sm leading-snug">
				{ticket.subject}
			</span>
			<span className="block truncate text-muted-foreground text-xs">
				{ticket.requester.name}
			</span>
			<span className="flex flex-wrap gap-1">
				<PriorityBadge priority={ticket.priority} />
				{ticket.displayFields.map((field) => (
					<CustomFieldChip
						key={field.key}
						fieldKey={field.key}
						value={field.value}
					/>
				))}
			</span>
			<span className="flex items-center justify-between gap-2 text-muted-foreground text-xs">
				<AssigneeStack assignee={ticket.assignee} />
				<span className="flex shrink-0 items-center gap-2 tabular-nums">
					{formatRelative(ticket.updatedAt)}
				</span>
			</span>
		</button>
	)
}
