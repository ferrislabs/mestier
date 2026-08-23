import type { TimeSpan } from '#/pages/planning/lib/amplitude'
import {
	buildAttendeeIndex,
	type CalendarAttendeeVM,
	type EventDetailVM,
} from '#/pages/planning/lib/build-calendar-model'
import {
	type CalendarFilter,
	type CalendarNature,
	entryNature,
	matchesFilter,
} from '#/pages/planning/lib/calendar-filters'
import {
	entryIsRecurring,
	entryLabel,
	entryMemberIds,
} from '#/pages/planning/lib/entries'
import { stackOverlapping } from '#/pages/planning/lib/layout'
import {
	entryOccursOnDate,
	minuteSpanOnDate,
} from '#/pages/planning/lib/occurrence'
import { enumerateDays } from '#/pages/planning/lib/window'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'

const DAYS_PER_WEEK = 7

/**
 * Number of timed entries a cell shows before switching to a counter. A month
 * cell has a fixed height: past that, we announce the remainder rather than
 * silently clipping it.
 */
export const MONTH_CELL_ENTRY_LIMIT = 4

export interface MonthEntryVM {
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	title: string
	/** Whether this task still follows a recurrence — see `entryIsRecurring`. */
	isRecurring: boolean
	/** Start time on that day, empty for an entry carried over from the day before. */
	timeLabel: string
	entry: PlanningEntry
	detail: EventDetailVM
}

/** Banner of an all-day entry, spread over the columns it covers. */
export interface MonthSpanVM {
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	title: string
	/** Whether this task still follows a recurrence — see `entryIsRecurring`. */
	isRecurring: boolean
	/** Starting column within the week, 0 = Monday. */
	startIndex: number
	/** Number of columns covered within this week. */
	length: number
	/** Vertical rank within the banner, so two entries never overlap. */
	lane: number
	/** The entry spills before the start / after the end of this week. */
	continuesBefore: boolean
	continuesAfter: boolean
	entry: PlanningEntry
	detail: EventDetailVM
}

export interface MonthDayVM {
	date: string
	/** Day number, or "1 sept." at a month change, the way Apple does it. */
	dayLabel: string
	isToday: boolean
	/** Day of a neighbouring month, dimmed to complete the week. */
	isOutsideMonth: boolean
	isWeekend: boolean
	entries: MonthEntryVM[]
	/** Timed entries left unshown for lack of room. */
	hiddenCount: number
}

export interface MonthWeekVM {
	days: MonthDayVM[]
	spans: MonthSpanVM[]
	/** Height of the all-day banner, in ranks. */
	laneCount: number
}

export interface MonthModel {
	weeks: MonthWeekVM[]
	weekdayLabels: string[]
	/** Entries dropped by the filters, across every day. */
	hiddenByFilter: number
}

export interface BuildMonthModelInput {
	from: string
	to: string
	/** Displayed month, as `YYYY-MM` — used to dim the neighbouring days. */
	month: string
	entries: PlanningEntry[]
	resources: PlanningResource[]
	timeZone: string
	today: string
	filter: CalendarFilter
	memberIds?: string[]
}

export function buildMonthModel(input: BuildMonthModelInput): MonthModel {
	const visible = input.entries.filter(
		(entry) =>
			matchesFilter(entry, input.filter) &&
			matchesMembers(entry, input.memberIds),
	)

	const days = enumerateDays(input.from, input.to)
	const attendeesByMember = buildAttendeeIndex(input.resources)
	const weeks: MonthWeekVM[] = []

	for (let index = 0; index < days.length; index += DAYS_PER_WEEK) {
		weeks.push(
			buildWeek({
				dates: days.slice(index, index + DAYS_PER_WEEK),
				entries: visible,
				month: input.month,
				timeZone: input.timeZone,
				today: input.today,
				attendeesByMember,
			}),
		)
	}

	return {
		weeks,
		weekdayLabels: days.slice(0, DAYS_PER_WEEK).map(formatWeekday),
		hiddenByFilter: input.entries.length - visible.length,
	}
}

function buildWeek(params: {
	dates: string[]
	entries: PlanningEntry[]
	month: string
	timeZone: string
	today: string
	attendeesByMember: Map<string, CalendarAttendeeVM>
}): MonthWeekVM {
	const spans = buildSpans(params)

	return {
		days: params.dates.map((date) =>
			buildDay({
				date,
				entries: params.entries,
				month: params.month,
				timeZone: params.timeZone,
				today: params.today,
				attendeesByMember: params.attendeesByMember,
			}),
		),
		spans,
		laneCount: spans.reduce((max, span) => Math.max(max, span.lane + 1), 0),
	}
}

function buildDay(params: {
	date: string
	entries: PlanningEntry[]
	month: string
	timeZone: string
	today: string
	attendeesByMember: Map<string, CalendarAttendeeVM>
}): MonthDayVM {
	const timed = params.entries
		.filter((entry) => !entry.all_day)
		.filter((entry) =>
			entryOccursOnDate(toTimeSpan(entry), params.date, params.timeZone),
		)
		.map((entry) => ({
			entry,
			startMinute: minuteSpanOnDate(
				toTimeSpan(entry),
				params.date,
				params.timeZone,
			).startMinute,
		}))
		.sort((a, b) => a.startMinute - b.startMinute)

	const shown = timed.slice(0, MONTH_CELL_ENTRY_LIMIT)

	return {
		date: params.date,
		dayLabel: formatDayLabel(params.date),
		isToday: params.date === params.today,
		isOutsideMonth: params.date.slice(0, 7) !== params.month,
		isWeekend: isWeekend(params.date),
		entries: shown.map(({ entry, startMinute }) => ({
			key: `${entry.id}-${params.date}`,
			entryId: entry.id,
			nature: entryNature(entry),
			title: entryLabel(entry),
			isRecurring: entryIsRecurring(entry),
			timeLabel: startsOnOrAfter(entry, params.date, params.timeZone)
				? formatMinute(startMinute)
				: '',
			entry,
			detail: buildDetail({
				entry,
				date: params.date,
				timeZone: params.timeZone,
				attendeesByMember: params.attendeesByMember,
			}),
		})),
		hiddenCount: timed.length - shown.length,
	}
}

function buildSpans(params: {
	dates: string[]
	entries: PlanningEntry[]
	timeZone: string
	attendeesByMember: Map<string, CalendarAttendeeVM>
}): MonthSpanVM[] {
	const first = params.dates[0]
	const last = params.dates.at(-1)
	if (!first || !last) return []

	const candidates = params.entries
		.filter((entry) => entry.all_day)
		.map((entry) => {
			const covered = params.dates
				.map((date, index) => ({ date, index }))
				.filter(({ date }) =>
					entryOccursOnDate(toTimeSpan(entry), date, params.timeZone),
				)
			return { entry, covered }
		})
		.filter(({ covered }) => covered.length > 0)
		.map(({ entry, covered }) => {
			const startIndex = covered[0]?.index ?? 0
			const endIndex = covered.at(-1)?.index ?? startIndex

			return {
				entry,
				startIndex,
				length: endIndex - startIndex + 1,
				continuesBefore: !startsOnOrAfter(entry, first, params.timeZone),
				continuesAfter: !endsOnOrBefore(entry, last, params.timeZone),
			}
		})
		.sort((a, b) => a.startIndex - b.startIndex || b.length - a.length)

	// Same rank spreading as timed overlaps: here the axis is the column index
	// instead of the minute.
	return stackOverlapping(candidates, (candidate) => ({
		startMinute: candidate.startIndex,
		endMinute: candidate.startIndex + candidate.length,
	})).map(({ item, row }) => ({
		key: `${item.entry.id}-${first}`,
		entryId: item.entry.id,
		nature: entryNature(item.entry),
		title: entryLabel(item.entry),
		isRecurring: entryIsRecurring(item.entry),
		startIndex: item.startIndex,
		length: item.length,
		lane: row,
		continuesBefore: item.continuesBefore,
		continuesAfter: item.continuesAfter,
		entry: item.entry,
		detail: buildDetail({
			entry: item.entry,
			date: params.dates[item.startIndex] ?? first,
			timeZone: params.timeZone,
			attendeesByMember: params.attendeesByMember,
		}),
	}))
}

/**
 * Describes an entry for the detail panel, in the format the week view already
 * produces — that shared contract is what lets both projections share the same
 * panel.
 */
function buildDetail(params: {
	entry: PlanningEntry
	date: string
	timeZone: string
	attendeesByMember: Map<string, CalendarAttendeeVM>
}): EventDetailVM {
	const span = minuteSpanOnDate(
		toTimeSpan(params.entry),
		params.date,
		params.timeZone,
	)

	return {
		title: entryLabel(params.entry),
		dateLabel: formatLongDate(params.date),
		timeLabel: params.entry.all_day
			? 'Journée entière'
			: `${formatMinute(span.startMinute)} – ${formatMinute(span.endMinute)}`,
		attendees: entryMemberIds(params.entry)
			.map((memberId) => params.attendeesByMember.get(memberId))
			.filter((attendee) => attendee !== undefined),
		entry: params.entry,
	}
}

function formatLongDate(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'long',
		day: 'numeric',
		month: 'long',
		timeZone: 'UTC',
	}).format(new Date(`${date}T00:00:00Z`))
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

function startsOnOrAfter(
	entry: PlanningEntry,
	date: string,
	timeZone: string,
): boolean {
	return !entryOccursOnDate(toTimeSpan(entry), previousDay(date), timeZone)
}

function endsOnOrBefore(
	entry: PlanningEntry,
	date: string,
	timeZone: string,
): boolean {
	return !entryOccursOnDate(toTimeSpan(entry), nextDay(date), timeZone)
}

function previousDay(date: string): string {
	return shiftIsoDay(date, -1)
}

function nextDay(date: string): string {
	return shiftIsoDay(date, 1)
}

function shiftIsoDay(date: string, delta: number): string {
	const shifted = new Date(`${date}T00:00:00Z`)
	shifted.setUTCDate(shifted.getUTCDate() + delta)
	return shifted.toISOString().slice(0, 10)
}

function toTimeSpan(entry: PlanningEntry): TimeSpan {
	return {
		startsAt: entry.starts_at,
		endsAt: entry.ends_at,
		allDay: entry.all_day,
	}
}

function formatMinute(minute: number): string {
	const hours = Math.floor(minute / 60)
	const minutes = minute % 60
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

function formatDayLabel(date: string): string {
	const day = Number(date.slice(8, 10))
	if (day !== 1) return String(day)

	const month = new Intl.DateTimeFormat('fr-FR', {
		month: 'short',
		timeZone: 'UTC',
	}).format(new Date(`${date}T00:00:00Z`))

	return `1 ${month}`
}

function formatWeekday(date: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'short',
		timeZone: 'UTC',
	})
		.format(new Date(`${date}T00:00:00Z`))
		.replace('.', '')
}

function isWeekend(date: string): boolean {
	const weekday = new Date(`${date}T00:00:00Z`).getUTCDay()
	return weekday === 0 || weekday === 6
}
