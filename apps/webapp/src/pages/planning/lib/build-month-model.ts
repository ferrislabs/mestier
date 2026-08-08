import type { TimeSpan } from '#/pages/planning/lib/amplitude'
import {
	type CalendarFilter,
	type CalendarNature,
	entryNature,
	matchesFilter,
} from '#/pages/planning/lib/calendar-filters'
import { entryEmployeeIds, entryLabel } from '#/pages/planning/lib/entries'
import { stackOverlapping } from '#/pages/planning/lib/layout'
import {
	entryOccursOnDate,
	minuteSpanOnDate,
} from '#/pages/planning/lib/occurrence'
import { enumerateDays } from '#/pages/planning/lib/window'
import type { PlanningEntry } from '#/pages/planning/types'

const DAYS_PER_WEEK = 7

/**
 * Nombre d'entrées horaires affichées dans une case avant de basculer sur un
 * compteur. Une case de mois a une hauteur fixe : au-delà, on annonce le reste
 * plutôt que de rogner silencieusement.
 */
export const MONTH_CELL_ENTRY_LIMIT = 4

export interface MonthEntryVM {
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	title: string
	/** Heure de début sur ce jour, vide pour une entrée qui vient de la veille. */
	timeLabel: string
	entry: PlanningEntry
}

/** Bandeau d'une entrée à la journée, étalé sur les colonnes qu'elle couvre. */
export interface MonthSpanVM {
	key: string
	entryId: string
	nature: CalendarNature | 'unknown'
	title: string
	/** Colonne de départ dans la semaine, 0 = lundi. */
	startIndex: number
	/** Nombre de colonnes couvertes dans cette semaine. */
	length: number
	/** Rang vertical dans le bandeau, pour ne pas superposer deux entrées. */
	lane: number
	/** L'entrée déborde avant le début / après la fin de cette semaine. */
	continuesBefore: boolean
	continuesAfter: boolean
	entry: PlanningEntry
}

export interface MonthDayVM {
	date: string
	/** Numéro du jour, ou « 1 sept. » au changement de mois, comme le fait Apple. */
	dayLabel: string
	isToday: boolean
	/** Jour d'un mois voisin, affiché en retrait pour compléter la semaine. */
	isOutsideMonth: boolean
	isWeekend: boolean
	entries: MonthEntryVM[]
	/** Entrées horaires non affichées faute de place. */
	hiddenCount: number
}

export interface MonthWeekVM {
	days: MonthDayVM[]
	spans: MonthSpanVM[]
	/** Hauteur du bandeau des journées entières, en rangs. */
	laneCount: number
}

export interface MonthModel {
	weeks: MonthWeekVM[]
	weekdayLabels: string[]
	/** Entrées écartées par les filtres, tous jours confondus. */
	hiddenByFilter: number
}

export interface BuildMonthModelInput {
	from: string
	to: string
	/** Mois affiché, au format `YYYY-MM` — sert à griser les jours voisins. */
	month: string
	entries: PlanningEntry[]
	timeZone: string
	today: string
	filter: CalendarFilter
	employeeIds?: string[]
}

export function buildMonthModel(input: BuildMonthModelInput): MonthModel {
	const visible = input.entries.filter(
		(entry) =>
			matchesFilter(entry, input.filter) &&
			matchesEmployees(entry, input.employeeIds),
	)

	const days = enumerateDays(input.from, input.to)
	const weeks: MonthWeekVM[] = []

	for (let index = 0; index < days.length; index += DAYS_PER_WEEK) {
		weeks.push(
			buildWeek({
				dates: days.slice(index, index + DAYS_PER_WEEK),
				entries: visible,
				month: input.month,
				timeZone: input.timeZone,
				today: input.today,
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
			timeLabel: startsOnOrAfter(entry, params.date, params.timeZone)
				? formatMinute(startMinute)
				: '',
			entry,
		})),
		hiddenCount: timed.length - shown.length,
	}
}

function buildSpans(params: {
	dates: string[]
	entries: PlanningEntry[]
	timeZone: string
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

	// Même répartition en rangs que les chevauchements horaires : l'axe est ici
	// l'index de colonne au lieu de la minute.
	return stackOverlapping(candidates, (candidate) => ({
		startMinute: candidate.startIndex,
		endMinute: candidate.startIndex + candidate.length,
	})).map(({ item, row }) => ({
		key: `${item.entry.id}-${first}`,
		entryId: item.entry.id,
		nature: entryNature(item.entry),
		title: entryLabel(item.entry),
		startIndex: item.startIndex,
		length: item.length,
		lane: row,
		continuesBefore: item.continuesBefore,
		continuesAfter: item.continuesAfter,
		entry: item.entry,
	}))
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
