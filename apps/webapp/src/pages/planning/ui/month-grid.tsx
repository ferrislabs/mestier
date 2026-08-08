import { cn } from '#/lib/utils'
import type {
	MonthDayVM,
	MonthEntryVM,
	MonthModel,
	MonthSpanVM,
	MonthWeekVM,
} from '#/pages/planning/lib/build-month-model'
import type { CalendarNature } from '#/pages/planning/lib/calendar-filters'

/** Hauteur d'un rang du bandeau des journées entières, en pixels. */
const SPAN_LANE_HEIGHT_PX = 22

export interface MonthGridProps {
	model: MonthModel
	onSelectEntry: (entry: MonthEntryVM['entry']) => void
}

/**
 * Vue mois en grille de semaines.
 *
 * Elle ne réutilise pas la grille horaire : sur trente jours, un axe des heures
 * donne trente colonnes illisibles. Ici une semaine est une ligne, un jour une
 * case, et chaque entrée une ligne de texte — on lit « qu'est-ce qui se passe
 * ce mois-ci », pas « à quelle heure ».
 */
export function MonthGrid({ model, onSelectEntry }: MonthGridProps) {
	return (
		<div className="overflow-x-auto">
			<div className="min-w-3xl">
				<div className="grid grid-cols-7 border-b">
					{model.weekdayLabels.map((label) => (
						<div
							key={label}
							className="px-2 py-2 text-center text-xs font-medium capitalize text-muted-foreground"
						>
							{label}
						</div>
					))}
				</div>

				{model.weeks.map((week) => (
					<WeekRow
						key={week.days[0]?.date}
						week={week}
						onSelectEntry={onSelectEntry}
					/>
				))}
			</div>
		</div>
	)
}

interface WeekRowProps {
	week: MonthWeekVM
	onSelectEntry: (entry: MonthEntryVM['entry']) => void
}

function WeekRow({ week, onSelectEntry }: WeekRowProps) {
	return (
		<div className="relative border-b last:border-b-0">
			<div className="grid min-h-32 grid-cols-7">
				{week.days.map((day) => (
					<DayCell
						key={day.date}
						day={day}
						spanOffset={week.laneCount * SPAN_LANE_HEIGHT_PX}
						onSelectEntry={onSelectEntry}
					/>
				))}
			</div>

			{/* Les bandeaux se superposent à la grille : une entrée à la journée
			    couvre plusieurs colonnes, elle ne peut donc pas vivre dans une case. */}
			{week.spans.map((span) => (
				<SpanBar key={span.key} span={span} onSelectEntry={onSelectEntry} />
			))}
		</div>
	)
}

interface DayCellProps {
	day: MonthDayVM
	/** Place réservée en haut de la case pour les bandeaux de la semaine. */
	spanOffset: number
	onSelectEntry: (entry: MonthEntryVM['entry']) => void
}

function DayCell({ day, spanOffset, onSelectEntry }: DayCellProps) {
	return (
		<div
			className={cn(
				'flex flex-col gap-0.5 border-l px-1 pb-1 first:border-l-0',
				day.isWeekend && 'bg-muted/30',
				day.isOutsideMonth && 'bg-muted/50',
			)}
		>
			<div className="flex justify-end px-1 pt-1">
				<span
					className={cn(
						'flex h-6 min-w-6 items-center justify-center rounded-full px-1.5 text-sm tabular-nums',
						day.isToday && 'bg-destructive font-semibold text-white',
						!day.isToday && day.isOutsideMonth && 'text-muted-foreground/60',
						!day.isToday && !day.isOutsideMonth && 'text-foreground',
					)}
				>
					{day.dayLabel}
				</span>
			</div>

			<div style={{ marginTop: spanOffset }} className="flex flex-col gap-0.5">
				{day.entries.map((entry) => (
					<EntryRow
						key={entry.key}
						entry={entry}
						onSelect={() => onSelectEntry(entry.entry)}
					/>
				))}
				{day.hiddenCount > 0 ? (
					<span className="px-1 text-[11px] font-medium text-muted-foreground">
						{day.hiddenCount} de plus
					</span>
				) : null}
			</div>
		</div>
	)
}

interface EntryRowProps {
	entry: MonthEntryVM
	onSelect: () => void
}

function EntryRow({ entry, onSelect }: EntryRowProps) {
	return (
		<button
			type="button"
			onClick={onSelect}
			className="flex w-full items-center gap-1.5 rounded px-1 py-px text-left transition-colors hover:bg-muted"
		>
			<span
				aria-hidden="true"
				className={cn(
					'h-3 w-1 shrink-0 rounded-full',
					natureDotClassName(entry.nature),
				)}
			/>
			<span className="min-w-0 flex-1 truncate text-[11px] text-foreground">
				{entry.title}
			</span>
			{entry.timeLabel ? (
				<span className="shrink-0 text-[11px] text-muted-foreground tabular-nums">
					{entry.timeLabel}
				</span>
			) : null}
		</button>
	)
}

interface SpanBarProps {
	span: MonthSpanVM
	onSelectEntry: (entry: MonthEntryVM['entry']) => void
}

function SpanBar({ span, onSelectEntry }: SpanBarProps) {
	const columnWidth = 100 / 7

	return (
		<button
			type="button"
			onClick={() => onSelectEntry(span.entry)}
			className={cn(
				'absolute truncate px-2 text-left text-[11px] font-medium leading-5',
				natureBarClassName(span.nature),
				span.continuesBefore ? 'rounded-l-none' : 'rounded-l-full',
				span.continuesAfter ? 'rounded-r-none' : 'rounded-r-full',
			)}
			style={{
				left: `calc(${span.startIndex * columnWidth}% + 4px)`,
				width: `calc(${span.length * columnWidth}% - 8px)`,
				// Sous le numéro du jour, qui occupe la première ligne de la case.
				top: 32 + span.lane * SPAN_LANE_HEIGHT_PX,
			}}
		>
			{span.title}
		</button>
	)
}

function natureDotClassName(nature: CalendarNature | 'unknown'): string {
	switch (nature) {
		case 'task':
			return 'bg-primary'
		case 'leave':
			return 'bg-success'
		case 'sick':
			return 'bg-warning'
		default:
			return 'bg-muted-foreground/50'
	}
}

function natureBarClassName(nature: CalendarNature | 'unknown'): string {
	switch (nature) {
		case 'task':
			return 'bg-brand-soft text-primary'
		case 'leave':
			return 'bg-success-soft text-success'
		case 'sick':
			return 'bg-warning-soft text-warning'
		default:
			return 'bg-muted text-muted-foreground'
	}
}
