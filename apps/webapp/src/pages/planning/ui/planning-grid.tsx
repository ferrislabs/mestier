import { X } from 'lucide-react'
import { cn } from '#/lib/utils'
import type { MinuteRange } from '#/pages/planning/lib/amplitude'
import {
	buildGridModel,
	type GridCellVM,
	type GridResourceRowVM,
} from '#/pages/planning/lib/build-grid-model'
import type { EntryTone } from '#/pages/planning/lib/entries'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningView,
	PlanningWorkTime,
} from '#/pages/planning/types'
import { CurrentTimeLine } from '#/pages/planning/ui/current-time-line'

/** The drag payload a task segment carries, read back on drop — see the planning design doc's "Drag & drop" section. */
interface DragPayload {
	entryId: string
	resourceId: string
	date: string
}

const DRAG_PAYLOAD_FORMAT = 'application/json'

export interface TaskDropEvent {
	entryId: string
	sourceResourceId: string
	sourceDate: string
	targetResourceId: string
	targetDate: string
}

export interface RemoveAssigneeEvent {
	entryId: string
	resourceId: string
}

export interface OpenTaskEvent {
	entryId: string
}

export interface PlanningGridProps {
	view: PlanningView
	windowFrom: string
	windowTo: string
	timeZone: string
	resources: PlanningResource[]
	entries: PlanningEntry[]
	workTime: PlanningWorkTime[]
	/** Forwarded to {@link CurrentTimeLine} — tests only, see its own doc. */
	now?: Date
	/** A task segment was dropped onto a (possibly different) row and/or day — see the planning design doc's "Drag & drop" decision: one event, whichever axis moved. */
	onDropTask?: (event: TaskDropEvent) => void
	/** The small "×" on a task segment — unassigns that one resource, through the same complete-list path a move uses. */
	onRemoveAssignee?: (event: RemoveAssigneeEvent) => void
	/** A task segment was clicked — opens its detail/edit sheet. Absence segments stay inert (see `PlanningGrid`'s own doc — absence management moved to the HR module), never call this. */
	onOpenTask?: (event: OpenTaskEvent) => void
}

const SEGMENT_HEIGHT_PX: Record<PlanningView, number> = {
	day: 26,
	week: 18,
	month: 10,
}
const CELL_PADDING_PX = 6

/**
 * The resource × time grid — day, week and month share this one component,
 * only the granularity of the time axis changes (see the planning design
 * doc's "une grille, trois granularités" section). Pure function of its
 * props: no hooks, no fetch. The single stateful, ticking element is
 * {@link CurrentTimeLine}, rendered once as an overlay, never per cell.
 *
 * Absence segments render but are inert — no click, no keyboard activation
 * — since absence management moved to the HR module (see the planning
 * remodel design doc's "Absences vers le module RH" decision); the grid
 * keeps displaying them, it no longer edits them.
 */
export function PlanningGrid({
	view,
	windowFrom,
	windowTo,
	timeZone,
	resources,
	entries,
	workTime,
	now,
	onDropTask,
	onRemoveAssignee,
	onOpenTask,
}: PlanningGridProps) {
	const model = buildGridModel({
		windowFrom,
		windowTo,
		timeZone,
		resources,
		entries,
		workTime,
	})

	if (resources.length === 0) {
		return (
			<div className="rounded-lg border bg-card p-8 text-center text-sm text-muted-foreground">
				Aucune ressource à afficher pour cette organisation.
			</div>
		)
	}

	return (
		<div
			className="flex overflow-hidden rounded-lg border bg-card"
			data-testid="planning-grid"
			data-view={view}
		>
			<div className="flex w-40 shrink-0 flex-col border-r sm:w-52">
				<div className="h-9 shrink-0 border-b" />
				{model.rows.map((row) => (
					<div
						key={row.resourceId}
						className="flex min-h-12 items-center truncate border-b px-3 py-2 text-sm font-medium last:border-b-0"
						title={row.displayName}
					>
						<span className="truncate">{row.displayName}</span>
					</div>
				))}
			</div>

			<div className="relative flex-1 overflow-x-auto">
				<TimeHeader
					view={view}
					amplitude={model.amplitude}
					columns={model.columns}
				/>

				<div className="relative">
					{model.rows.map((row) => (
						<GridRow
							key={row.resourceId}
							view={view}
							row={row}
							onDropTask={onDropTask}
							onRemoveAssignee={onRemoveAssignee}
							onOpenTask={onOpenTask}
						/>
					))}

					<div className="pointer-events-none absolute inset-x-0 top-9 bottom-0">
						<CurrentTimeLine
							timeZone={timeZone}
							windowFrom={windowFrom}
							windowTo={windowTo}
							amplitude={model.amplitude}
							orientation={view === 'day' ? 'horizontal' : 'vertical'}
							columns={view === 'day' ? undefined : model.columns}
							now={now}
						/>
					</div>
				</div>
			</div>
		</div>
	)
}

function TimeHeader({
	view,
	amplitude,
	columns,
}: {
	view: PlanningView
	amplitude: MinuteRange
	columns: string[]
}) {
	if (view === 'day') {
		return (
			<div className="relative h-9 shrink-0 border-b">
				{hourMarks(amplitude).map((minute) => (
					<span
						key={minute}
						className="absolute top-2 -translate-x-1/2 text-xs text-muted-foreground"
						style={{ left: `${percentOf(minute, amplitude)}%` }}
					>
						{formatHourLabel(minute)}
					</span>
				))}
			</div>
		)
	}

	return (
		<div className="flex h-9 shrink-0 border-b">
			{columns.map((date) => (
				<div
					key={date}
					className="flex min-w-0 flex-1 items-center justify-center text-xs font-medium text-muted-foreground"
				>
					{formatColumnLabel(date, view)}
				</div>
			))}
		</div>
	)
}

interface GridInteractionHandlers {
	onDropTask?: (event: TaskDropEvent) => void
	onRemoveAssignee?: (event: RemoveAssigneeEvent) => void
	onOpenTask?: (event: OpenTaskEvent) => void
}

function GridRow({
	view,
	row,
	onDropTask,
	onRemoveAssignee,
	onOpenTask,
}: GridInteractionHandlers & {
	view: PlanningView
	row: GridResourceRowVM
}) {
	const maxRowCount = Math.max(1, ...row.cells.map((cell) => cell.rowCount))
	const height = maxRowCount * SEGMENT_HEIGHT_PX[view] + CELL_PADDING_PX * 2

	return (
		<div
			className="flex border-b last:border-b-0"
			style={{ minHeight: `${Math.max(height, 48)}px` }}
			data-testid="grid-row"
			data-resource-id={row.resourceId}
		>
			{row.cells.map((cell) => (
				<GridCell
					key={cell.date}
					view={view}
					cell={cell}
					resourceId={row.resourceId}
					onDropTask={onDropTask}
					onRemoveAssignee={onRemoveAssignee}
					onOpenTask={onOpenTask}
				/>
			))}
		</div>
	)
}

function GridCell({
	view,
	cell,
	resourceId,
	onDropTask,
	onRemoveAssignee,
	onOpenTask,
}: GridInteractionHandlers & {
	view: PlanningView
	cell: GridCellVM
	resourceId: string
}) {
	const segmentHeight = SEGMENT_HEIGHT_PX[view]

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: HTML5 drag & drop's drop target has no native interactive equivalent — it isn't a keyboard-operable widget, dragging a segment onto it is.
		<div
			className={cn(
				'relative min-w-0 flex-1 border-r py-1.5 last:border-r-0',
				view === 'month' && cell.hasAbsence && 'bg-warning-soft/60',
			)}
			data-testid="grid-cell"
			data-date={cell.date}
			data-has-absence={cell.hasAbsence}
			onDragOver={(event) => {
				event.preventDefault()
			}}
			onDrop={(event) => {
				event.preventDefault()
				const raw = event.dataTransfer.getData(DRAG_PAYLOAD_FORMAT)
				if (!raw) return
				const source = JSON.parse(raw) as DragPayload
				onDropTask?.({
					entryId: source.entryId,
					sourceResourceId: source.resourceId,
					sourceDate: source.date,
					targetResourceId: resourceId,
					targetDate: cell.date,
				})
			}}
		>
			{cell.backgroundBars.map((bar, index) => (
				<div
					// biome-ignore lint/suspicious/noArrayIndexKey: background bars have no stable id, order is stable within a render
					key={index}
					className="absolute inset-y-1 rounded-sm bg-muted"
					style={{ left: `${bar.left}%`, width: `${bar.width}%` }}
				/>
			))}

			{cell.segments.map((segment) => {
				const isTask = segment.tone === 'task'
				const openable = isTask && Boolean(onOpenTask)

				return (
					// biome-ignore lint/a11y/noStaticElementInteractions: the drag source has no native interactive equivalent; absence segments are display-only (see PlanningGrid's own doc — absence management moved to the HR module). A task segment's own click/keyboard affordance is added explicitly below (role="button", tabIndex, onKeyDown) when onOpenTask is provided.
					<div
						key={segment.entryId}
						className={cn(
							'group absolute overflow-hidden rounded-sm px-1 text-[10px] leading-tight text-nowrap',
							toneClassName(segment.tone),
							view === 'day' ? 'text-xs' : 'text-[9px]',
							openable && 'cursor-pointer',
						)}
						data-testid="grid-segment"
						data-entry-id={segment.entryId}
						data-tone={segment.tone}
						title={segment.label}
						style={{
							left: `${segment.left}%`,
							width: `${Math.max(segment.width, 2)}%`,
							top: `${segment.row * segmentHeight}px`,
							height: `${segmentHeight - 2}px`,
						}}
						draggable={isTask}
						onDragStart={
							isTask
								? (event) => {
										const payload: DragPayload = {
											entryId: segment.entryId,
											resourceId,
											date: cell.date,
										}
										event.dataTransfer.setData(
											DRAG_PAYLOAD_FORMAT,
											JSON.stringify(payload),
										)
									}
								: undefined
						}
						role={openable ? 'button' : undefined}
						tabIndex={openable ? 0 : undefined}
						onClick={
							openable
								? () => onOpenTask?.({ entryId: segment.entryId })
								: undefined
						}
						onKeyDown={
							openable
								? (event) => {
										if (event.key === 'Enter' || event.key === ' ') {
											event.preventDefault()
											onOpenTask?.({ entryId: segment.entryId })
										}
									}
								: undefined
						}
					>
						{view === 'day' ? segment.label : null}
						{segment.labels.length > 0 ? (
							<span className="absolute inset-x-0.5 bottom-0.5 flex gap-0.5">
								{segment.labels.slice(0, 4).map((label) => (
									<span
										key={label.id}
										className="size-1 rounded-full ring-1 ring-white/70"
										style={{ backgroundColor: label.color }}
										title={label.name}
									/>
								))}
							</span>
						) : null}
						{isTask && onRemoveAssignee ? (
							<button
								type="button"
								className="absolute top-0 right-0 hidden size-3.5 items-center justify-center rounded-bl-sm bg-black/20 hover:bg-black/40 group-hover:flex"
								title="Retirer cette personne du chantier"
								aria-label="Retirer cette personne du chantier"
								onClick={(event) => {
									event.stopPropagation()
									onRemoveAssignee({ entryId: segment.entryId, resourceId })
								}}
							>
								<X className="size-2.5" />
							</button>
						) : null}
					</div>
				)
			})}
		</div>
	)
}

function toneClassName(tone: EntryTone): string {
	switch (tone) {
		case 'task':
			return 'bg-primary text-primary-foreground'
		case 'absence':
			return 'bg-warning-soft text-warning border border-warning/40'
		default:
			return 'bg-muted text-muted-foreground border border-dashed border-border'
	}
}

function hourMarks(amplitude: MinuteRange): number[] {
	const marks: number[] = []
	for (
		let minute = amplitude.startMinute;
		minute <= amplitude.endMinute;
		minute += 60
	) {
		marks.push(minute)
	}
	return marks
}

function percentOf(minute: number, amplitude: MinuteRange): number {
	const span = amplitude.endMinute - amplitude.startMinute
	if (span <= 0) return 0
	return ((minute - amplitude.startMinute) / span) * 100
}

function formatHourLabel(minute: number): string {
	return `${String(Math.floor(minute / 60)).padStart(2, '0')}:00`
}

function formatColumnLabel(date: string, view: PlanningView): string {
	const parsed = new Date(`${date}T00:00:00Z`)
	const options: Intl.DateTimeFormatOptions =
		view === 'month'
			? { day: 'numeric', timeZone: 'UTC' }
			: { weekday: 'short', day: 'numeric', timeZone: 'UTC' }
	return new Intl.DateTimeFormat('fr-FR', options).format(parsed)
}
