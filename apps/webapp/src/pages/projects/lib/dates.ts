import type { IsoDate } from '#/pages/projects/lib/model'

const ISO_DATE = /^(\d{4})-(\d{2})-(\d{2})$/
const DAY_MS = 86_400_000

export function todayIso(now: Date = new Date()): IsoDate {
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${now.getFullYear()}-${month}-${day}`
}

export function isIsoDate(value: unknown): value is IsoDate {
	if (typeof value !== 'string') return false
	const match = ISO_DATE.exec(value)
	if (!match) return false
	const date = new Date(Date.UTC(+match[1], +match[2] - 1, +match[3]))
	return date.toISOString().slice(0, 10) === value
}

function toUtc(iso: IsoDate): number {
	const [year, month, day] = iso.split('-').map(Number)
	return Date.UTC(year, month - 1, day)
}

export function addDays(iso: IsoDate, days: number): IsoDate {
	return new Date(toUtc(iso) + days * DAY_MS).toISOString().slice(0, 10)
}

export function diffDays(from: IsoDate, to: IsoDate): number {
	return Math.round((toUtc(to) - toUtc(from)) / DAY_MS)
}

export function yearOf(iso: IsoDate): number {
	return Number(iso.slice(0, 4))
}

export function monthOf(iso: IsoDate): string {
	return iso.slice(0, 7)
}

const DATE_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	year: 'numeric',
	timeZone: 'UTC',
})

const SHORT_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	timeZone: 'UTC',
})

const MONTH_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC',
})

export function formatDate(iso: IsoDate): string {
	return DATE_FORMAT.format(new Date(toUtc(iso)))
}

export function formatShortDate(iso: IsoDate): string {
	return SHORT_FORMAT.format(new Date(toUtc(iso)))
}

export function formatMonth(month: string): string {
	return MONTH_FORMAT.format(new Date(toUtc(`${month}-01`)))
}

export function formatDateTime(timestamp: string): string {
	const moment = new Date(timestamp)
	if (Number.isNaN(moment.getTime())) return timestamp
	const hours = String(moment.getHours()).padStart(2, '0')
	const minutes = String(moment.getMinutes()).padStart(2, '0')
	return `${formatDate(todayIso(moment))}, ${hours}:${minutes}`
}
