const FULL = new Intl.NumberFormat('fr-FR', {
	style: 'currency',
	currency: 'EUR',
})

const ROUNDED = new Intl.NumberFormat('fr-FR', {
	style: 'currency',
	currency: 'EUR',
	maximumFractionDigits: 0,
})

export function formatMoney(cents: number): string {
	return FULL.format(cents / 100)
}

export function formatMoneyRounded(cents: number): string {
	return ROUNDED.format(cents / 100)
}

const DAY = 86_400_000

function zoneOf(value: string): 'UTC' | undefined {
	return value.length === 10 ? 'UTC' : undefined
}

export function formatDate(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: 'numeric',
		month: 'short',
		year: 'numeric',
		timeZone: zoneOf(value),
	}).format(new Date(value))
}

export function formatShortDate(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: 'numeric',
		month: 'short',
		timeZone: zoneOf(value),
	}).format(new Date(value))
}

export function formatMonth(month: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		month: 'long',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(new Date(`${month}-01`))
}

export function formatDateTime(timestamp: string): string {
	const moment = new Date(timestamp)
	if (Number.isNaN(moment.getTime())) return timestamp
	const hours = String(moment.getHours()).padStart(2, '0')
	const minutes = String(moment.getMinutes()).padStart(2, '0')
	return `${formatDate(todayIso(moment))}, ${hours}:${minutes}`
}

export function todayIso(now: Date = new Date()): string {
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${now.getFullYear()}-${month}-${day}`
}

export function addDays(iso: string, days: number): string {
	return new Date(new Date(iso).getTime() + days * DAY)
		.toISOString()
		.slice(0, 10)
}

export function monthOf(iso: string): string {
	return iso.slice(0, 7)
}

export function centsToInput(cents: number): string {
	const sign = cents < 0 ? '-' : ''
	const abs = Math.abs(cents)
	const euros = Math.floor(abs / 100)
	const rest = abs % 100
	return rest === 0
		? `${sign}${euros}`
		: `${sign}${euros},${String(rest).padStart(2, '0')}`
}

export function parseAmount(input: string): number | null {
	const cleaned = input
		.replace(/[\s\u00a0\u202f€]/g, '')
		.replace(',', '.')
		.trim()
	if (!/^-?\d+(\.\d{1,2})?$/.test(cleaned)) return null
	return Math.round(Number(cleaned) * 100)
}
