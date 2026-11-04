import { GripVertical, Loader2 } from 'lucide-react'
import { type KeyboardEvent, useEffect, useRef } from 'react'
import { StatusBadge } from '#/components/ui/surface'
import { supportStatusTone } from '#/pages/support/lib/columns'
import type { SupportBoardUIProps, SupportCardVM } from '#/pages/support/types'

const KEYBOARD_HINT_ID = 'support-card-keyboard-hint'

export type TicketColumnsProps = Pick<
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
>

export function TicketColumns({
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
}: TicketColumnsProps) {
	return (
		<div className="min-w-0 max-w-full overflow-x-auto overscroll-x-contain pb-2">
			<p id={KEYBOARD_HINT_ID} className="sr-only">
				Flèches gauche et droite pour changer de colonne.
			</p>
			<div className="grid w-max auto-cols-[18rem] grid-flow-col gap-3">
				{columns.map((column) => (
					<section
						key={column.status}
						aria-label={`Colonne ${column.label}`}
						className="flex min-h-[28rem] min-w-0 flex-col overflow-hidden rounded-lg border bg-card"
						onDragOver={(event) => event.preventDefault()}
						onDrop={() => onDropOnColumn(column.status)}
					>
						<div className="flex items-center justify-between gap-2 border-b px-3 py-3">
							<p className="min-w-0 truncate text-sm font-semibold">
								{column.label}
							</p>
							<StatusBadge tone={supportStatusTone(column.status)}>
								{column.cards.length}
							</StatusBadge>
						</div>

						<div className="flex flex-1 flex-col gap-2 bg-muted/25 p-2">
							{isLoading ? (
								<div className="flex flex-1 items-center justify-center rounded-md border border-dashed bg-card/70 p-4 text-center text-xs text-muted-foreground">
									Chargement…
								</div>
							) : column.cards.length === 0 ? (
								<div className="flex flex-1 items-center justify-center rounded-md border border-dashed bg-card/70 p-4 text-center text-xs text-muted-foreground">
									Aucun ticket
								</div>
							) : (
								column.cards.map((card) => (
									<TicketCard
										key={card.id}
										card={card}
										columnLabel={column.label}
										canMove={canMove}
										isDragging={draggedTicketId === card.id}
										isMoving={movingTicketId === card.id}
										shouldFocus={focusedTicketId === card.id}
										onFocus={() => onFocusCard(card.id)}
										onDragStart={() => onDragStart(card.id)}
										onDragEnd={onDragEnd}
										onShiftColumn={(direction) =>
											onShiftColumn(card.id, direction)
										}
									/>
								))
							)}
						</div>
					</section>
				))}
			</div>
		</div>
	)
}

interface TicketCardProps {
	card: SupportCardVM
	columnLabel: string
	canMove: boolean
	isDragging: boolean
	isMoving: boolean
	shouldFocus: boolean
	onFocus: () => void
	onDragStart: () => void
	onDragEnd: () => void
	onShiftColumn: (direction: 1 | -1) => void
}

function TicketCard({
	card,
	columnLabel,
	canMove,
	isDragging,
	isMoving,
	shouldFocus,
	onFocus,
	onDragStart,
	onDragEnd,
	onShiftColumn,
}: TicketCardProps) {
	const handleReference = useRef<HTMLButtonElement>(null)

	useEffect(() => {
		if (shouldFocus) handleReference.current?.focus()
	}, [shouldFocus])

	function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
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
		<article
			aria-label={`${card.subject}, colonne ${columnLabel}`}
			draggable={canMove ? true : undefined}
			onDragStart={canMove ? onDragStart : undefined}
			onDragEnd={canMove ? onDragEnd : undefined}
			className={`rounded-md border bg-card p-3 shadow-xs transition ${
				isDragging ? 'opacity-50 ring-2 ring-primary/30' : 'hover:shadow-md'
			}`}
		>
			<div className="flex items-start gap-2">
				{canMove ? (
					<button
						type="button"
						ref={handleReference}
						aria-label={`Déplacer ${card.subject}`}
						aria-describedby={KEYBOARD_HINT_ID}
						onKeyDown={handleKeyDown}
						onFocus={onFocus}
						className="mt-0.5 shrink-0 cursor-grab rounded text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						<GripVertical aria-hidden="true" className="size-4" />
					</button>
				) : null}
				<div className="min-w-0 flex-1">
					<p className="text-xs font-medium text-muted-foreground">
						{card.numberLabel}
					</p>
					<p className="mt-0.5 text-sm font-semibold break-words">
						{card.subject}
					</p>
					<p className="mt-0.5 truncate text-xs text-muted-foreground">
						{card.assigneeLabel}
					</p>
				</div>
				{isMoving ? (
					<span className="mt-0.5 shrink-0">
						<Loader2
							aria-hidden="true"
							className="size-4 animate-spin text-muted-foreground"
						/>
						<span className="sr-only">Déplacement en cours</span>
					</span>
				) : null}
			</div>

			{card.badges.length > 0 ? (
				<div className="mt-2 flex flex-wrap gap-1">
					{card.badges.map((badge) => (
						<StatusBadge key={badge.label} tone={badge.tone}>
							{badge.label}
						</StatusBadge>
					))}
				</div>
			) : null}

			{card.fields.length > 0 ? (
				<dl className="mt-3 flex flex-col gap-1 text-xs">
					{card.fields.map((field) => (
						<div key={field.key} className="flex items-baseline gap-1.5">
							<dt className="shrink-0 text-muted-foreground">{field.key}</dt>
							<dd className="min-w-0 break-words">{field.value}</dd>
						</div>
					))}
				</dl>
			) : null}
		</article>
	)
}
