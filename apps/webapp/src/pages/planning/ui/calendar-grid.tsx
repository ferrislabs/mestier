import { ChevronDown } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import type { MinuteRange } from '#/pages/planning/lib/amplitude'
import type {
	CalendarDayVM,
	CalendarEventVM,
	CalendarModel,
} from '#/pages/planning/lib/build-calendar-model'
import { formatHourLabel } from '#/pages/planning/lib/build-calendar-model'
import type { CalendarNature } from '#/pages/planning/lib/calendar-filters'
import {
	currentTimePercent,
	isCurrentTimeVisible,
	millisecondsUntilNextMinute,
} from '#/pages/planning/lib/current-time'
import { EventDetailCard } from '#/pages/planning/ui/event-detail-card'

/** Ce que le calendrier sait faire d'un événement — le reste vit dans la feature. */
export interface CalendarEventCallbacks {
	/** Ouvre l'entrée dans son écran complet : fiche de tâche, formulaire d'absence. */
	onOpen: (event: CalendarEventVM) => void
	onChangeStatus?: (event: CalendarEventVM, status: Schemas.TaskStatus) => void
	onDelete?: (event: CalendarEventVM) => void
	isPending?: boolean
}

/** Hauteur d'une heure de grille, en pixels. Fixe : c'est elle qui donne son échelle au calendrier. */
const HOUR_HEIGHT_PX = 64

const GUTTER_CLASS = 'w-16 shrink-0 md:w-20'

export interface CalendarGridProps {
	model: CalendarModel
	callbacks: CalendarEventCallbacks
	/**
	 * Fige l'heure courante. La production l'omet — le trait avance seul ;
	 * les tests passent un instant fixe, comme {@link CurrentTimeLine}.
	 */
	now?: Date
}

export function CalendarGrid({ model, callbacks, now }: CalendarGridProps) {
	const hourCount = Math.max(
		1,
		(model.amplitude.endMinute - model.amplitude.startMinute) / 60,
	)
	const bodyHeight = hourCount * HOUR_HEIGHT_PX
	const hasAllDay = model.days.some((day) => day.allDayEvents.length > 0)
	const scrollRef = useRef<HTMLDivElement>(null)
	const { startMinute, endMinute } = model.amplitude
	const scrollToMinute = model.scrollToMinute

	// La grille couvre les 24 h : on l'ouvre sur la première entrée de la période
	// plutôt qu'à minuit, sinon l'écran s'ouvre sur des heures vides. On
	// repositionne au changement de période, pas à chaque rendu.
	useEffect(() => {
		const container = scrollRef.current
		if (!container) return

		const span = Math.max(endMinute - startMinute, 1)
		const ratio = (scrollToMinute - startMinute) / span
		container.scrollTop = ratio * bodyHeight
	}, [scrollToMinute, startMinute, endMinute, bodyHeight])

	return (
		<div className="overflow-x-auto">
			<div className="min-w-3xl">
				<DayHeaderRow days={model.days} />

				{hasAllDay ? (
					<AllDayRow days={model.days} callbacks={callbacks} />
				) : null}

				<div
					ref={scrollRef}
					className="relative max-h-[65vh] overflow-y-auto overscroll-contain"
				>
					<div className="relative flex" style={{ height: bodyHeight }}>
						<HourGutter
							marks={model.hourMarks}
							amplitude={model.amplitude}
							height={bodyHeight}
						/>

						<div className="relative flex flex-1">
							<OffHoursBands
								amplitude={model.amplitude}
								workingRange={model.workingRange}
							/>
							<HourLines marks={model.hourMarks} amplitude={model.amplitude} />

							{model.days.map((day) => (
								<DayColumn key={day.date} day={day} callbacks={callbacks} />
							))}

							<NowLine model={model} controlledNow={now} gutterOffset={false} />
						</div>

						<NowLine model={model} controlledNow={now} gutterOffset />
					</div>
				</div>
			</div>
		</div>
	)
}

function DayHeaderRow({ days }: { days: CalendarDayVM[] }) {
	return (
		<div className="flex border-b">
			<div className={GUTTER_CLASS} />
			{days.map((day) => (
				<div
					key={day.date}
					className={cn(
						'flex-1 border-l px-2 py-3 text-center',
						day.isToday && 'bg-brand-soft/60',
					)}
				>
					<p
						className={cn(
							'text-xs font-medium capitalize',
							day.isToday ? 'text-primary' : 'text-muted-foreground',
						)}
					>
						{day.weekdayLabel}
					</p>
					<p
						className={cn(
							'mt-0.5 text-2xl font-normal tabular-nums',
							day.isToday ? 'text-primary' : 'text-foreground',
						)}
					>
						{day.dayLabel}
					</p>
					{day.isToday ? (
						<span className="mx-auto mt-2 block h-0.5 w-10 rounded-full bg-primary" />
					) : null}
				</div>
			))}
		</div>
	)
}

interface AllDayRowProps {
	days: CalendarDayVM[]
	callbacks: CalendarEventCallbacks
}

function AllDayRow({ days, callbacks }: AllDayRowProps) {
	return (
		<div className="flex border-b bg-muted/30">
			<div
				className={cn(
					GUTTER_CLASS,
					'py-2 pr-2 text-right text-[11px] font-medium text-muted-foreground',
				)}
			>
				Journée
			</div>
			{days.map((day) => (
				<div
					key={day.date}
					className={cn(
						'flex flex-1 flex-col gap-1 border-l p-1.5',
						day.isToday && 'bg-brand-soft/40',
					)}
				>
					{day.allDayEvents.map((event) => (
						<EventPopover key={event.key} event={event} callbacks={callbacks}>
							<button
								type="button"
								className={cn(
									'w-full truncate rounded-lg px-2 py-1 text-left text-[11px] font-medium transition-shadow hover:shadow-sm',
									natureClassName(event.nature),
								)}
							>
								{event.title}
							</button>
						</EventPopover>
					))}
				</div>
			))}
		</div>
	)
}

interface HourGutterProps {
	marks: number[]
	amplitude: MinuteRange
	height: number
}

function HourGutter({ marks, amplitude, height }: HourGutterProps) {
	// Même repère que les traits d'heures : une position proportionnelle à
	// l'amplitude, et non l'index de la marque — sinon les libellés se décalent
	// dès que l'amplitude ne commence pas sur une heure pleine.
	const span = Math.max(amplitude.endMinute - amplitude.startMinute, 1)

	return (
		<div className={cn(GUTTER_CLASS, 'relative')} style={{ height }}>
			{marks.map((minute, index) => (
				<span
					key={minute}
					className={cn(
						'absolute right-2 text-[11px] font-medium text-muted-foreground tabular-nums',
						// Centrer sur le trait ferait déborder la première étiquette
						// au-dessus de la grille — sur le bandeau des journées entières —
						// et la dernière en dessous. Les deux bornes s'alignent donc sur
						// le bord au lieu d'être centrées.
						index === 0 && 'translate-y-0',
						index > 0 && index < marks.length - 1 && '-translate-y-1/2',
						index === marks.length - 1 && '-translate-y-full',
					)}
					style={{
						top: `${((minute - amplitude.startMinute) / span) * 100}%`,
					}}
				>
					{formatHourLabel(minute)}
				</span>
			))}
		</div>
	)
}

interface HourLinesProps {
	marks: number[]
	amplitude: MinuteRange
}

function HourLines({ marks, amplitude }: HourLinesProps) {
	const span = Math.max(amplitude.endMinute - amplitude.startMinute, 1)

	return (
		<div aria-hidden="true" className="pointer-events-none absolute inset-0">
			{marks.map((minute) => (
				<span
					key={minute}
					className="absolute inset-x-0 border-t border-border/70"
					style={{
						top: `${((minute - amplitude.startMinute) / span) * 100}%`,
					}}
				/>
			))}
		</div>
	)
}

interface OffHoursBandsProps {
	amplitude: MinuteRange
	workingRange: MinuteRange
}

/** Assombrit ce qui tombe hors des heures travaillées, pour situer la journée dans les 24 h. */
function OffHoursBands({ amplitude, workingRange }: OffHoursBandsProps) {
	const span = Math.max(amplitude.endMinute - amplitude.startMinute, 1)
	const toPercent = (minute: number) =>
		((minute - amplitude.startMinute) / span) * 100

	const beforeHeight = Math.max(toPercent(workingRange.startMinute), 0)
	const afterTop = Math.min(toPercent(workingRange.endMinute), 100)

	return (
		<div aria-hidden="true" className="pointer-events-none absolute inset-0">
			{beforeHeight > 0 ? (
				<span
					className="absolute inset-x-0 top-0 bg-muted/40"
					style={{ height: `${beforeHeight}%` }}
				/>
			) : null}
			{afterTop < 100 ? (
				<span
					className="absolute inset-x-0 bottom-0 bg-muted/40"
					style={{ top: `${afterTop}%` }}
				/>
			) : null}
		</div>
	)
}

interface DayColumnProps {
	day: CalendarDayVM
	callbacks: CalendarEventCallbacks
}

function DayColumn({ day, callbacks }: DayColumnProps) {
	return (
		<div
			className={cn(
				'relative flex-1 border-l',
				day.isWeekend && 'bg-muted/20',
				day.isToday && 'bg-brand-soft/25',
			)}
		>
			{day.timedEvents.map((event) => (
				<EventCard key={event.key} event={event} callbacks={callbacks} />
			))}
		</div>
	)
}

interface EventCardProps {
	event: CalendarEventVM
	callbacks: CalendarEventCallbacks
}

/**
 * Densité de la carte selon la durée qu'elle couvre. Une tâche d'une demi-heure
 * n'a pas la place d'afficher ses participants : plutôt que de tronquer au
 * hasard, les rangs disparaissent par ordre d'importance décroissante.
 */
const FULL_CARD_MINUTES = 90
const TIMED_CARD_MINUTES = 45

function EventCard({ event, callbacks }: EventCardProps) {
	const width = 100 / event.columnCount
	// Un chevauchement décale la carte dans sa colonne plutôt que de la
	// rétrécir davantage : deux cartes côte à côte restent lisibles.
	const left = width * event.column
	const showFooter = event.durationMinutes >= FULL_CARD_MINUTES
	const showTime = event.durationMinutes >= TIMED_CARD_MINUTES

	return (
		<EventPopover event={event} callbacks={callbacks}>
			<button
				type="button"
				className={cn(
					'group absolute flex flex-col overflow-hidden rounded-xl px-2.5 py-2 text-left transition-shadow hover:shadow-md data-[state=open]:shadow-md data-[state=open]:ring-2 data-[state=open]:ring-current/30',
					natureClassName(event.nature),
				)}
				style={{
					top: `${event.top}%`,
					height: `${event.height}%`,
					left: `${left}%`,
					width: `calc(${width}% - 6px)`,
					marginLeft: '3px',
				}}
			>
				<p className="truncate text-[13px] font-semibold leading-tight">
					{event.title}
				</p>
				{showTime ? (
					<p className="mt-0.5 truncate text-[11px] font-medium opacity-70 tabular-nums">
						{event.timeLabel}
					</p>
				) : null}

				{showFooter ? (
					<span className="mt-auto flex items-end justify-between gap-2">
						{event.attendees.length > 0 ? (
							<AttendeeStack attendees={event.attendees} />
						) : (
							<span />
						)}
						<ChevronDown className="size-3.5 shrink-0 opacity-0 transition-opacity group-hover:opacity-70" />
					</span>
				) : null}
			</button>
		</EventPopover>
	)
}

interface EventPopoverProps {
	event: CalendarEventVM
	callbacks: CalendarEventCallbacks
	children: React.ReactNode
}

/**
 * Panneau de détail d'un événement, ancré sur sa carte.
 *
 * Un clic ouvre ce que l'entrée contient déjà — horaire, assignés, client,
 * étiquettes, description — au lieu d'envoyer vers un écran pour le lire. Les
 * actions de fond restent derrière « ouvrir ».
 */
function EventPopover({ event, callbacks, children }: EventPopoverProps) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent
				align="start"
				side="right"
				className="w-96 rounded-2xl p-5"
			>
				<EventDetailCard
					event={event}
					isPending={callbacks.isPending}
					onOpen={() => {
						setOpen(false)
						callbacks.onOpen(event)
					}}
					onClose={() => setOpen(false)}
					onChangeStatus={
						callbacks.onChangeStatus
							? (status) => callbacks.onChangeStatus?.(event, status)
							: undefined
					}
					onDelete={
						callbacks.onDelete
							? () => {
									setOpen(false)
									callbacks.onDelete?.(event)
								}
							: undefined
					}
				/>
			</PopoverContent>
		</Popover>
	)
}

function AttendeeStack({
	attendees,
}: {
	attendees: CalendarEventVM['attendees']
}) {
	const shown = attendees.slice(0, 3)
	const extra = attendees.length - shown.length

	return (
		<span className="mt-1.5 flex items-center -space-x-1.5">
			{shown.map((attendee) => (
				<span
					key={attendee.id}
					title={attendee.name}
					className="flex size-5 items-center justify-center rounded-full bg-card text-[9px] font-semibold text-foreground ring-1 ring-border"
				>
					{attendee.initials}
				</span>
			))}
			{extra > 0 ? (
				<span className="flex size-5 items-center justify-center rounded-full bg-card text-[9px] font-semibold text-muted-foreground ring-1 ring-border">
					+{extra}
				</span>
			) : null}
		</span>
	)
}

interface NowLineProps {
	model: CalendarModel
	controlledNow: Date | undefined
	/** Rendu dans la gouttière : la pastille d'heure, sans le trait. */
	gutterOffset: boolean
}

/**
 * Trait de l'heure courante. Seul élément du calendrier qui ne dérive pas des
 * données serveur : il porte son propre `now`, isolé ici pour que la grille
 * reste une fonction pure de ses props.
 */
function NowLine({ model, controlledNow, gutterOffset }: NowLineProps) {
	const [now, setNow] = useState<Date>(() => controlledNow ?? new Date())

	useEffect(() => {
		if (controlledNow) {
			setNow(controlledNow)
			return
		}

		let intervalId: ReturnType<typeof setInterval> | undefined
		const timeoutId = setTimeout(() => {
			setNow(new Date())
			intervalId = setInterval(() => setNow(new Date()), 60_000)
		}, millisecondsUntilNextMinute(new Date()))

		return () => {
			clearTimeout(timeoutId)
			if (intervalId !== undefined) clearInterval(intervalId)
		}
	}, [controlledNow])

	const days = model.days
	const first = days[0]
	const last = days.at(-1)
	if (!first || !last) return null

	const visible = isCurrentTimeVisible({
		now,
		timeZone: model.timeZone,
		windowFrom: first.date,
		windowTo: last.date,
		amplitude: model.amplitude,
	})
	if (!visible) return null

	const percent = currentTimePercent(now, model.timeZone, model.amplitude)

	if (gutterOffset) {
		return (
			<span
				data-testid="calendar-now-label"
				className="pointer-events-none absolute left-0 z-20 -translate-y-1/2 rounded-full bg-destructive px-1.5 py-0.5 text-[10px] font-semibold text-white tabular-nums"
				style={{ top: `${percent}%` }}
			>
				{formatClock(now, model.timeZone)}
			</span>
		)
	}

	return (
		<span
			data-testid="calendar-now-line"
			aria-hidden="true"
			className="pointer-events-none absolute inset-x-0 z-10 border-t-2 border-destructive"
			style={{ top: `${percent}%` }}
		/>
	)
}

function formatClock(now: Date, timeZone: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		hour: '2-digit',
		minute: '2-digit',
		timeZone,
	}).format(now)
}

export function natureClassName(nature: CalendarNature | 'unknown'): string {
	switch (nature) {
		case 'task':
			return 'bg-brand-soft text-primary'
		case 'leave':
			return 'bg-success-soft text-success'
		case 'sick':
			return 'bg-warning-soft text-warning'
		case 'unavailable':
			return 'bg-muted text-muted-foreground'
		default:
			return 'bg-muted text-muted-foreground'
	}
}
