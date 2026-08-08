import {
	computeAmplitude,
	FALLBACK_AMPLITUDE,
	MINUTES_PER_DAY,
	type MinuteInterval,
	type MinuteRange,
	type TimeSpan,
} from '#/pages/planning/lib/amplitude'
import {
	type CalendarFilter,
	type CalendarNature,
	entryNature,
	matchesFilter,
} from '#/pages/planning/lib/calendar-filters'
import { entryEmployeeIds, entryLabel } from '#/pages/planning/lib/entries'
import {
	computeSegmentPosition,
	type MinuteSpan,
	stackOverlapping,
} from '#/pages/planning/lib/layout'
import {
	entryOccursOnDate,
	minuteSpanOnDate,
} from '#/pages/planning/lib/occurrence'
import { enumerateDays } from '#/pages/planning/lib/window'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningWorkTime,
} from '#/pages/planning/types'

export interface CalendarAttendeeVM {
	id: string
	name: string
	initials: string
}

export interface CalendarEventVM {
	/** Identifiant de rendu : une entrée s'étalant sur plusieurs jours produit un segment par jour. */
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	title: string
	timeLabel: string
	allDay: boolean
	/** Minutes depuis minuit local, pour positionner le défilement d'ouverture. */
	startMinute: number
	/** Durée du segment sur ce jour — c'est elle qui décide de la densité de la carte. */
	durationMinutes: number
	/** Jour du segment, en toutes lettres, pour l'en-tête du panneau de détail. */
	dateLabel: string
	/** Position verticale, en pourcentage de l'amplitude horaire visible. */
	top: number
	height: number
	/** Répartition horizontale des chevauchements dans la colonne du jour. */
	column: number
	columnCount: number
	attendees: CalendarAttendeeVM[]
	entry: PlanningEntry
}

export interface CalendarDayVM {
	date: string
	weekdayLabel: string
	dayLabel: string
	isToday: boolean
	isWeekend: boolean
	/** Entrées sur la journée entière, rendues dans un bandeau au-dessus de la grille. */
	allDayEvents: CalendarEventVM[]
	timedEvents: CalendarEventVM[]
}

export interface CalendarModel {
	days: CalendarDayVM[]
	amplitude: MinuteRange
	hourMarks: number[]
	timeZone: string
	/** Nombre d'entrées écartées par le filtre courant, pour le dire à l'utilisateur. */
	hiddenCount: number
	/** Minute sur laquelle ouvrir le défilement vertical. */
	scrollToMinute: number
	/**
	 * Plage travaillée, déduite des horaires des employés — à défaut, la journée
	 * type. Sur une grille de 24 h, c'est elle qui distingue d'un coup d'œil les
	 * heures ouvrées du reste.
	 */
	workingRange: MinuteRange
}

export interface BuildCalendarModelInput {
	from: string
	to: string
	entries: PlanningEntry[]
	resources: PlanningResource[]
	workTime: PlanningWorkTime[]
	timeZone: string
	today: string
	filter: CalendarFilter
	/** Employés retenus ; vide ou absent = toute l'équipe. */
	employeeIds?: string[]
}

export function buildCalendarModel(
	input: BuildCalendarModelInput,
): CalendarModel {
	const visible = input.entries.filter(
		(entry) =>
			matchesFilter(entry, input.filter) &&
			matchesEmployees(entry, input.employeeIds),
	)
	const amplitude = CALENDAR_AMPLITUDE
	const attendeesByEmployee = buildAttendeeIndex(input.resources)

	const days = enumerateDays(input.from, input.to).map((date) =>
		buildDay({
			date,
			entries: visible,
			amplitude,
			attendeesByEmployee,
			timeZone: input.timeZone,
			today: input.today,
		}),
	)

	return {
		days,
		amplitude,
		hourMarks: hourMarks(amplitude),
		timeZone: input.timeZone,
		hiddenCount: input.entries.length - visible.length,
		scrollToMinute: firstEventMinute(days),
		workingRange: computeAmplitude(
			[],
			flattenWorkTime(input.workTime),
			input.timeZone,
		),
	}
}

function buildDay(params: {
	date: string
	entries: PlanningEntry[]
	amplitude: MinuteRange
	attendeesByEmployee: Map<string, CalendarAttendeeVM>
	timeZone: string
	today: string
}): CalendarDayVM {
	const onThisDay = params.entries.filter((entry) =>
		entryOccursOnDate(toTimeSpan(entry), params.date, params.timeZone),
	)
	const allDay = onThisDay.filter((entry) => entry.all_day)
	const timed = onThisDay.filter((entry) => !entry.all_day)

	const stacked = stackOverlapping(timed, (entry) =>
		minuteSpanOnDate(toTimeSpan(entry), params.date, params.timeZone),
	)
	const columnCount = stacked.reduce(
		(max, item) => Math.max(max, item.row + 1),
		1,
	)

	return {
		date: params.date,
		weekdayLabel: formatWeekday(params.date),
		dayLabel: formatDayNumber(params.date),
		isToday: params.date === params.today,
		isWeekend: isWeekend(params.date),
		allDayEvents: allDay.map((entry) =>
			toEventVM({
				entry,
				date: params.date,
				amplitude: params.amplitude,
				attendeesByEmployee: params.attendeesByEmployee,
				timeZone: params.timeZone,
				column: 0,
				columnCount: 1,
			}),
		),
		timedEvents: stacked.map((item) =>
			toEventVM({
				entry: item.item,
				date: params.date,
				amplitude: params.amplitude,
				attendeesByEmployee: params.attendeesByEmployee,
				timeZone: params.timeZone,
				column: item.row,
				columnCount,
			}),
		),
	}
}

function toEventVM(params: {
	entry: PlanningEntry
	date: string
	amplitude: MinuteRange
	attendeesByEmployee: Map<string, CalendarAttendeeVM>
	timeZone: string
	column: number
	columnCount: number
}): CalendarEventVM {
	const span = minuteSpanOnDate(
		toTimeSpan(params.entry),
		params.date,
		params.timeZone,
	)
	// `computeSegmentPosition` raisonne sur un axe quelconque : ici l'axe est
	// vertical, donc `left`/`width` deviennent `top`/`height`.
	const position = computeSegmentPosition(span, params.amplitude)

	return {
		key: `${params.entry.id}-${params.date}`,
		entryId: params.entry.id,
		nature: entryNature(params.entry),
		title: entryLabel(params.entry),
		timeLabel: formatSpanLabel(span, params.entry.all_day),
		allDay: params.entry.all_day,
		startMinute: span.startMinute,
		durationMinutes: span.endMinute - span.startMinute,
		dateLabel: formatLongDate(params.date),
		top: position.left,
		height: position.width,
		column: params.column,
		columnCount: params.columnCount,
		attendees: entryEmployeeIds(params.entry)
			.map((employeeId) => params.attendeesByEmployee.get(employeeId))
			.filter((attendee) => attendee !== undefined),
		entry: params.entry,
	}
}

function matchesEmployees(
	entry: PlanningEntry,
	employeeIds: string[] | undefined,
): boolean {
	if (!employeeIds || employeeIds.length === 0) return true
	const ids = entryEmployeeIds(entry)
	if (ids.length === 0) return false
	return ids.some((id) => employeeIds.includes(id))
}

function buildAttendeeIndex(
	resources: PlanningResource[],
): Map<string, CalendarAttendeeVM> {
	const index = new Map<string, CalendarAttendeeVM>()

	for (const resource of resources) {
		if (!resource.employee_id) continue
		index.set(resource.employee_id, {
			id: resource.employee_id,
			name: resource.display_name,
			initials: initialsOf(resource.display_name),
		})
	}

	return index
}

export function initialsOf(name: string): string {
	return (
		name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('') || '?'
	)
}

function toTimeSpan(entry: PlanningEntry): TimeSpan {
	return {
		startsAt: entry.starts_at,
		endsAt: entry.ends_at,
		allDay: entry.all_day,
	}
}

function flattenWorkTime(workTime: PlanningWorkTime[]): MinuteInterval[] {
	return workTime.flatMap((employee) =>
		employee.days.flatMap((day) =>
			day.intervals.map((interval) => ({
				startsMinute: interval.starts_minute,
				endsMinute: interval.ends_minute,
			})),
		),
	)
}

/**
 * Le calendrier rend la journée entière, de minuit à minuit.
 *
 * La grille équipe se resserre au plus juste autour des entrées — utile quand
 * chaque employé n'a qu'une ligne. Un calendrier, lui, garde un cadre fixe :
 * une amplitude qui varie selon le contenu déplace les repères d'un jour à
 * l'autre, et masque les créneaux libres qu'on vient précisément y lire. La
 * hauteur qui en découle se parcourt au défilement, positionné d'emblée sur la
 * première entrée de la période.
 */
export const CALENDAR_AMPLITUDE: MinuteRange = {
	startMinute: 0,
	endMinute: MINUTES_PER_DAY,
}

/** Heure sur laquelle ouvrir la vue quand la période ne contient rien. */
const DEFAULT_SCROLL_MINUTE = FALLBACK_AMPLITUDE.startMinute

/** Ouvre la vue une demi-heure avant la première entrée, pour ne pas la coller en haut. */
const SCROLL_MARGIN_MINUTES = 30

/**
 * Première minute occupée de la période, marge comprise — sinon l'heure de
 * travail par défaut. Les entrées à la journée n'entrent pas dans le calcul :
 * elles vivent dans le bandeau, pas dans la grille horaire.
 */
function firstEventMinute(days: CalendarDayVM[]): number {
	const starts = days.flatMap((day) =>
		day.timedEvents.map((event) => event.startMinute),
	)
	if (starts.length === 0) return DEFAULT_SCROLL_MINUTE

	return Math.max(0, Math.min(...starts) - SCROLL_MARGIN_MINUTES)
}

/** Une marque par heure pleine dans l'amplitude, bornes comprises. */
export function hourMarks(amplitude: MinuteRange): number[] {
	const marks: number[] = []
	const first = Math.ceil(amplitude.startMinute / 60) * 60

	for (let minute = first; minute <= amplitude.endMinute; minute += 60) {
		marks.push(minute)
	}

	return marks
}

export function formatHourLabel(minute: number): string {
	const hours = Math.floor(minute / 60) % 24
	return `${String(hours).padStart(2, '0')}:00`
}

function formatSpanLabel(span: MinuteSpan, allDay: boolean): string {
	if (allDay) return 'Journée entière'
	return `${formatMinute(span.startMinute)} – ${formatMinute(span.endMinute)}`
}

function formatMinute(minute: number): string {
	const hours = Math.floor(minute / 60)
	const minutes = minute % 60
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

function formatLongDate(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'long',
		day: 'numeric',
		month: 'long',
		timeZone: 'UTC',
	}).format(new Date(`${date}T00:00:00Z`))
}

function formatWeekday(date: string): string {
	const label = new Intl.DateTimeFormat('fr-FR', {
		weekday: 'short',
		timeZone: 'UTC',
	}).format(new Date(`${date}T00:00:00Z`))

	return label.replace('.', '')
}

function formatDayNumber(date: string): string {
	return date.slice(8, 10)
}

function isWeekend(date: string): boolean {
	const weekday = new Date(`${date}T00:00:00Z`).getUTCDay()
	return weekday === 0 || weekday === 6
}
