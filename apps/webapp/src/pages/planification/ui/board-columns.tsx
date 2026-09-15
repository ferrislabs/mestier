import { parseISO } from 'date-fns'
import { CalendarIcon, GripVertical, Loader2 } from 'lucide-react'
import { type KeyboardEvent, useEffect, useRef } from 'react'
import { Button } from '#/components/ui/button'
import { Calendar } from '#/components/ui/calendar'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { StatusBadge } from '#/components/ui/surface'
import type { TaskStatus } from '#/pages/planification/lib/board'
import {
	calendarSelectionToDateRange,
	dateRangeToCalendarSelection,
} from '#/pages/planning/lib/task-form'

/** One shared instruction for every handle, rather than the same sentence repeated into each card's label. */
const KEYBOARD_HINT_ID = 'board-card-keyboard-hint'

export interface BoardCardLabelVM {
	id: string
	name: string
	color: string
}

export interface BoardCardVM {
	id: string
	title: string
	/** The card's window, already formatted — `null` when the task carries no dates. */
	windowLabel: string | null
	/** The same window as plain `yyyy-MM-dd` bounds, for the date chip's calendar. */
	dateRange: { startDate: string; endDate: string } | null
	assigneeLabel: string
	labels: BoardCardLabelVM[]
}

export interface BoardColumnVM {
	status: TaskStatus
	label: string
	cards: BoardCardVM[]
}

export interface BoardColumnsProps {
	columns: BoardColumnVM[]
	canMove: boolean
	isLoading: boolean
	draggedTaskId: string | null
	/** The card whose `PATCH` is in flight — greys out its controls, it does not hide the optimistic move. */
	movingTaskId: string | null
	/**
	 * The card that should hold DOM focus. A keyboard move re-parents the
	 * card into another column, which unmounts and remounts it; without this
	 * the user would land back on `document.body` after every arrow press.
	 */
	focusedTaskId: string | null
	onFocusCard: (taskId: string) => void
	onDragStart: (taskId: string) => void
	onDragEnd: () => void
	/** A drop on the column itself — no position named. */
	onDropOnColumn: (status: TaskStatus) => void
	/** A drop on one of the column's cards — the dragged card lands just before it. */
	onDropOnCard: (status: TaskStatus, targetTaskId: string) => void
	/** Left/right arrow: one column over, appended. */
	onShiftColumn: (taskId: string, direction: 1 | -1) => void
	/** Up/down arrow: one slot inside the same column. */
	onReorderCard: (taskId: string, direction: 1 | -1) => void
	onOpenCard: (taskId: string) => void
	onChangeCardWindow: (
		taskId: string,
		range: { startDate: string; endDate: string } | null,
	) => void
}

/**
 * The five columns and their cards — native HTML5 drag & drop, no library,
 * following `pages/customers/ui/customer-pipeline-board.tsx`, which is this
 * codebase's canonical board: `onDragOver` + `onDrop` on the column
 * `<section>`, `draggable` + `onDragStart` on the card, an `aria-label` and
 * a count badge per column, and a spelled-out empty state rather than a bare
 * frame.
 *
 * It departs from that file in one place: the pipeline board has no keyboard
 * path at all, and #466 requires one. A card is focusable, and the arrow
 * keys move it — left/right across columns, up/down inside one — issuing
 * exactly the requests the equivalent drop issues.
 */
export function BoardColumns({
	columns,
	canMove,
	isLoading,
	draggedTaskId,
	movingTaskId,
	focusedTaskId,
	onFocusCard,
	onDragStart,
	onDragEnd,
	onDropOnColumn,
	onDropOnCard,
	onShiftColumn,
	onReorderCard,
	onOpenCard,
	onChangeCardWindow,
}: BoardColumnsProps) {
	return (
		<div className="min-w-0 max-w-full overflow-x-auto overscroll-x-contain pb-2">
			<p id={KEYBOARD_HINT_ID} className="sr-only">
				Flèches gauche et droite pour changer de colonne, haut et bas pour
				réordonner la carte.
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
							<StatusBadge tone={columnTone(column.status)}>
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
									Aucune carte
								</div>
							) : (
								column.cards.map((card) => (
									<BoardCard
										key={card.id}
										card={card}
										columnLabel={column.label}
										canMove={canMove}
										isDragging={draggedTaskId === card.id}
										isMoving={movingTaskId === card.id}
										shouldFocus={focusedTaskId === card.id}
										onFocus={() => onFocusCard(card.id)}
										onDragStart={() => onDragStart(card.id)}
										onDragEnd={onDragEnd}
										onDrop={() => onDropOnCard(column.status, card.id)}
										onShiftColumn={(direction) =>
											onShiftColumn(card.id, direction)
										}
										onReorder={(direction) => onReorderCard(card.id, direction)}
										onOpen={() => onOpenCard(card.id)}
										onChangeWindow={(range) =>
											onChangeCardWindow(card.id, range)
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

interface BoardCardProps {
	card: BoardCardVM
	columnLabel: string
	canMove: boolean
	isDragging: boolean
	isMoving: boolean
	shouldFocus: boolean
	onFocus: () => void
	onDragStart: () => void
	onDragEnd: () => void
	onDrop: () => void
	onShiftColumn: (direction: 1 | -1) => void
	onReorder: (direction: 1 | -1) => void
	onOpen: () => void
	onChangeWindow: (range: { startDate: string; endDate: string } | null) => void
}

function BoardCard({
	card,
	columnLabel,
	canMove,
	isDragging,
	isMoving,
	shouldFocus,
	onFocus,
	onDragStart,
	onDragEnd,
	onDrop,
	onShiftColumn,
	onReorder,
	onOpen,
	onChangeWindow,
}: BoardCardProps) {
	const handleRef = useRef<HTMLButtonElement>(null)

	// Presentation only, and the reason `ui/` holds a hook at all here: a
	// keyboard move remounts this card under a different column, so focus has
	// to be put back on its handle or every arrow press would drop the user
	// onto the body.
	useEffect(() => {
		if (shouldFocus) handleRef.current?.focus()
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
		} else if (event.key === 'ArrowUp') {
			event.preventDefault()
			onReorder(-1)
		} else if (event.key === 'ArrowDown') {
			event.preventDefault()
			onReorder(1)
		}
	}

	return (
		<article
			aria-label={`${card.title}, colonne ${columnLabel}`}
			draggable={canMove}
			onDragStart={onDragStart}
			onDragEnd={onDragEnd}
			onDragOver={(event) => event.preventDefault()}
			onDrop={(event) => {
				event.stopPropagation()
				onDrop()
			}}
			className={`group rounded-md border bg-card p-3 shadow-xs transition ${
				isDragging ? 'opacity-50 ring-2 ring-primary/30' : 'hover:shadow-md'
			}`}
		>
			<div className="flex items-start gap-2">
				{canMove ? (
					<button
						type="button"
						ref={handleRef}
						aria-label={`Déplacer ${card.title}`}
						aria-describedby={KEYBOARD_HINT_ID}
						onKeyDown={handleKeyDown}
						onFocus={onFocus}
						className="mt-0.5 shrink-0 cursor-grab rounded text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						<GripVertical aria-hidden="true" className="size-4" />
					</button>
				) : null}
				<button
					type="button"
					className="min-w-0 flex-1 text-left outline-none"
					onClick={onOpen}
				>
					<p className="truncate text-sm font-semibold">{card.title}</p>
					<p className="mt-0.5 truncate text-xs text-muted-foreground">
						{card.assigneeLabel}
					</p>
				</button>
				{isMoving ? (
					<Loader2 className="mt-0.5 size-4 shrink-0 animate-spin text-muted-foreground" />
				) : null}
			</div>

			{card.labels.length > 0 ? (
				<div className="mt-2 flex flex-wrap gap-1">
					{card.labels.map((label) => (
						<span
							key={label.id}
							className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium text-white"
							style={{ backgroundColor: label.color }}
						>
							{label.name}
						</span>
					))}
				</div>
			) : null}

			<div className="mt-3 flex items-center justify-between gap-2">
				<BoardCardDateChip
					card={card}
					canMove={canMove}
					isMoving={isMoving}
					onChangeWindow={onChangeWindow}
				/>
			</div>
		</article>
	)
}

interface BoardCardDateChipProps {
	card: BoardCardVM
	canMove: boolean
	isMoving: boolean
	onChangeWindow: (range: { startDate: string; endDate: string } | null) => void
}

/**
 * Scheduling a card without leaving the board — the one control on the card
 * that writes dates, and the only one. It never sends a `status`: a card
 * that jumped columns because someone gave it a date is the failure #466
 * names explicitly.
 *
 * Writes an all-day window. A task that needs a time of day is edited in the
 * task sheet, which the card's title opens.
 */
function BoardCardDateChip({
	card,
	canMove,
	isMoving,
	onChangeWindow,
}: BoardCardDateChipProps) {
	const label = card.windowLabel ?? 'Sans date'

	if (!canMove) {
		return (
			<span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
				<CalendarIcon className="size-3.5" />
				{label}
			</span>
		)
	}

	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="ghost"
					size="sm"
					disabled={isMoving}
					className="h-7 px-2 text-xs font-normal text-muted-foreground"
					aria-label={`Dates de ${card.title} : ${label}`}
				>
					<CalendarIcon />
					{label}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="w-auto p-0" align="start">
				<Calendar
					mode="range"
					captionLayout="dropdown"
					defaultMonth={
						card.dateRange ? parseISO(card.dateRange.startDate) : undefined
					}
					selected={
						card.dateRange
							? dateRangeToCalendarSelection(
									card.dateRange.startDate,
									card.dateRange.endDate,
								)
							: undefined
					}
					onSelect={(selection) => {
						const next = calendarSelectionToDateRange(selection)
						if (next) onChangeWindow(next)
					}}
				/>
				{card.dateRange ? (
					<div className="border-t p-2">
						<Button
							type="button"
							variant="ghost"
							size="sm"
							className="w-full"
							onClick={() => onChangeWindow(null)}
						>
							Retirer les dates
						</Button>
					</div>
				) : null}
			</PopoverContent>
		</Popover>
	)
}

function columnTone(status: TaskStatus) {
	if (status === 'DONE') return 'success'
	if (status === 'CANCELLED') return 'error'
	if (status === 'IN_PROGRESS') return 'brand'
	if (status === 'PLANNED') return 'warning'
	return 'neutral'
}
