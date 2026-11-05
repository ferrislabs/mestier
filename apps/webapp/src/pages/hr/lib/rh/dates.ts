import type { IsoDate, Weekday } from '#/pages/hr/lib/rh/types'

const DAY_MS = 86_400_000

export const WEEKDAY_LABELS = [
	'Lundi',
	'Mardi',
	'Mercredi',
	'Jeudi',
	'Vendredi',
	'Samedi',
	'Dimanche',
] as const

export const WEEKDAY_SHORT = ['Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam', 'Dim']

const SHORT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	timeZone: 'UTC',
})
const LONG = new Intl.DateTimeFormat('fr-FR', {
	weekday: 'long',
	day: 'numeric',
	month: 'long',
	timeZone: 'UTC',
})
const FULL = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC',
})
const MONTH = new Intl.DateTimeFormat('fr-FR', {
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC',
})

export function parseIso(iso: IsoDate): Date {
	const [year, month, day] = iso.split('-').map(Number)
	return new Date(Date.UTC(year, month - 1, day))
}

export function isIsoDate(value: unknown): value is IsoDate {
	return (
		typeof value === 'string' &&
		/^\d{4}-\d{2}-\d{2}$/.test(value) &&
		toIso(parseIso(value)) === value
	)
}

export function toIso(date: Date): IsoDate {
	return date.toISOString().slice(0, 10)
}

export function todayIso(now: Date = new Date()): IsoDate {
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${now.getFullYear()}-${month}-${day}`
}

export function addDays(iso: IsoDate, days: number): IsoDate {
	return toIso(new Date(parseIso(iso).getTime() + days * DAY_MS))
}

export function diffDays(from: IsoDate, to: IsoDate): number {
	return Math.round(
		(parseIso(to).getTime() - parseIso(from).getTime()) / DAY_MS,
	)
}

export function weekdayOf(iso: IsoDate): Weekday {
	return ((parseIso(iso).getUTCDay() + 6) % 7) as Weekday
}

export function startOfWeek(iso: IsoDate): IsoDate {
	return addDays(iso, -weekdayOf(iso))
}

export function startOfMonth(iso: IsoDate): IsoDate {
	return `${iso.slice(0, 7)}-01`
}

export function endOfMonth(iso: IsoDate): IsoDate {
	const date = parseIso(iso)
	return toIso(
		new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0)),
	)
}

export function addMonths(iso: IsoDate, months: number): IsoDate {
	const date = parseIso(iso)
	return toIso(
		new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + months, 1)),
	)
}

export function shiftMonths(iso: IsoDate, months: number): IsoDate {
	const date = parseIso(iso)
	const target = new Date(
		Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + months, 1),
	)
	const lastDay = new Date(
		Date.UTC(target.getUTCFullYear(), target.getUTCMonth() + 1, 0),
	).getUTCDate()
	return toIso(
		new Date(
			Date.UTC(
				target.getUTCFullYear(),
				target.getUTCMonth(),
				Math.min(date.getUTCDate(), lastDay),
			),
		),
	)
}

export function daysInRange(start: IsoDate, end: IsoDate): IsoDate[] {
	const count = diffDays(start, end)
	return count < 0
		? []
		: Array.from({ length: count + 1 }, (_, index) => addDays(start, index))
}

export function formatShort(iso: IsoDate): string {
	return SHORT.format(parseIso(iso))
}

export function formatLong(iso: IsoDate): string {
	return LONG.format(parseIso(iso))
}

export function formatFull(iso: IsoDate): string {
	return FULL.format(parseIso(iso))
}

export function formatMonth(iso: IsoDate): string {
	return MONTH.format(parseIso(iso))
}

export function formatRange(start: IsoDate, end: IsoDate): string {
	return start === end
		? formatShort(start)
		: `${formatShort(start)} au ${formatShort(end)}`
}

export function formatDays(days: number): string {
	const label = Number.isInteger(days)
		? String(days)
		: days.toFixed(1).replace('.', ',')
	return `${label} ${Math.abs(days) > 1 ? 'jours' : 'jour'}`
}

export function formatDaysCompact(days: number): string {
	return Number.isInteger(days)
		? String(days)
		: days.toFixed(1).replace('.', ',')
}

export function yearsBetween(from: IsoDate, to: IsoDate): number {
	const months =
		(Number(to.slice(0, 4)) - Number(from.slice(0, 4))) * 12 +
		Number(to.slice(5, 7)) -
		Number(from.slice(5, 7)) -
		(to.slice(8, 10) < from.slice(8, 10) ? 1 : 0)
	return Math.max(0, Math.floor(months / 12))
}
