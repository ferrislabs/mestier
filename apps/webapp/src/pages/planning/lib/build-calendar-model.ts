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
import { entryLabel, entryMemberIds } from '#/pages/planning/lib/entries'
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

/**
 * What the detail panel needs, and nothing more. The week and month views
 * position their entries differently, but they describe them the same way —
 * that shared contract is what lets them share the panel.
 */
export interface EventDetailVM {
	title: string
	/** The entry's day, spelled out. */
	dateLabel: string
	/** Time range, or "Journée entière". */
	timeLabel: string
	attendees: CalendarAttendeeVM[]
	entry: PlanningEntry
}

export interface CalendarEventVM extends EventDetailVM {
	/** Render id: an entry spanning several days yields one segment per day. */
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	allDay: boolean
	/** Minutes since local midnight, to position the opening scroll. */
	startMinute: number
	/** The segment's duration on that day — it decides the card's density. */
	durationMinutes: number
	/** Vertical position, as a percentage of the visible time amplitude. */
	top: number
	height: number
	/** Horizontal spread of overlaps within the day's column. */
	column: number
	columnCount: number
}

export interface CalendarDayVM {
	date: string
	weekdayLabel: string
	dayLabel: string
	isToday: boolean
	isWeekend: boolean
	/** All-day entries, rendered in a banner above the grid. */
	allDayEvents: CalendarEventVM[]
	timedEvents: CalendarEventVM[]
}

export interface CalendarModel {
	days: CalendarDayVM[]
	amplitude: MinuteRange
	hourMarks: number[]
	timeZone: string
	/** Number of entries dropped by the current filter, to tell the user. */
	hiddenCount: number
	/** Minute the vertical scroll opens on. */
	scrollToMinute: number
	/**
	 * Worked range, inferred from the members' hours — a typical day when
	 * there are none. On a 24 h grid, it is what tells working hours from the
	 * rest at a glance.
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
	/** Selected members; empty or absent = the whole team. */
	memberIds?: string[]
}

export function buildCalendarModel(
	input: BuildCalendarModelInput,
): CalendarModel {
	const visible = input.entries.filter(
		(entry) =>
			matchesFilter(entry, input.filter) &&
			matchesMembers(entry, input.memberIds),
	)
	const amplitude = CALENDAR_AMPLITUDE
	const attendeesByMember = buildAttendeeIndex(input.resources)

	const days = enumerateDays(input.from, input.to).map((date) =>
		buildDay({
			date,
			entries: visible,
			amplitude,
			attendeesByMember,
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
	attendeesByMember: Map<string, CalendarAttendeeVM>
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
				attendeesByMember: params.attendeesByMember,
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
				attendeesByMember: params.attendeesByMember,
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
	attendeesByMember: Map<string, CalendarAttendeeVM>
	timeZone: string
	column: number
	columnCount: number
}): CalendarEventVM {
	const span = minuteSpanOnDate(
		toTimeSpan(params.entry),
		params.date,
		params.timeZone,
	)
	// `computeSegmentPosition` reasons on an arbitrary axis: here the axis is
	// vertical, so `left`/`width` become `top`/`height`.
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
		attendees: entryMemberIds(params.entry)
			.map((memberId) => params.attendeesByMember.get(memberId))
			.filter((attendee) => attendee !== undefined),
		entry: params.entry,
	}
}

function matchesMembers(
	entry: PlanningEntry,
	memberIds: string[] | undefined,
): boolean {
	if (!memberIds || memberIds.length === 0) return true
	const ids = entryMemberIds(entry)
	if (ids.length === 0) return false
	return ids.some((id) => memberIds.includes(id))
}

export function buildAttendeeIndex(
	resources: PlanningResource[],
): Map<string, CalendarAttendeeVM> {
	const index = new Map<string, CalendarAttendeeVM>()

	for (const resource of resources) {
		index.set(resource.member_id, {
			id: resource.member_id,
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
	return workTime.flatMap((member) =>
		member.days.flatMap((day) =>
			day.intervals.map((interval) => ({
				startsMinute: interval.starts_minute,
				endsMinute: interval.ends_minute,
			})),
		),
	)
}

/**
 * The calendar renders the whole day, midnight to midnight.
 *
 * The team grid tightens as closely as it can around the entries — useful when
 * each member has a single row. A calendar keeps a fixed frame instead: an
 * amplitude varying with the content shifts the landmarks from one day to the
 * next, and hides the free slots one comes here precisely to read. The height
 * that follows is walked by scrolling, positioned from the start on the
 * period's first entry.
 */
export const CALENDAR_AMPLITUDE: MinuteRange = {
	startMinute: 0,
	endMinute: MINUTES_PER_DAY,
}

/** Hour the view opens on when the period holds nothing. */
const DEFAULT_SCROLL_MINUTE = FALLBACK_AMPLITUDE.startMinute

/** Opens the view half an hour before the first entry, so it is not glued to the top. */
const SCROLL_MARGIN_MINUTES = 30

/**
 * First occupied minute of the period, margin included — otherwise the default
 * working hour. All-day entries stay out of the computation: they live in the
 * banner, not in the time grid.
 */
function firstEventMinute(days: CalendarDayVM[]): number {
	const starts = days.flatMap((day) =>
		day.timedEvents.map((event) => event.startMinute),
	)
	if (starts.length === 0) return DEFAULT_SCROLL_MINUTE

	return Math.max(0, Math.min(...starts) - SCROLL_MARGIN_MINUTES)
}

/** One tick per whole hour within the amplitude, bounds included. */
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
