const TIME = new Intl.DateTimeFormat('fr-FR', {
	hour: '2-digit',
	minute: '2-digit',
})
const DAY = new Intl.DateTimeFormat('fr-FR', { day: 'numeric', month: 'short' })
const FULL = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	year: 'numeric',
	hour: '2-digit',
	minute: '2-digit',
})

const MINUTE = 60_000
const HOUR = 60 * MINUTE

function sameDay(a: Date, b: Date): boolean {
	return (
		a.getFullYear() === b.getFullYear() &&
		a.getMonth() === b.getMonth() &&
		a.getDate() === b.getDate()
	)
}

export function formatRelative(iso: string, now: Date = new Date()): string {
	const date = new Date(iso)
	const elapsed = now.getTime() - date.getTime()
	if (elapsed < MINUTE) return 'à l’instant'
	if (elapsed < HOUR) return `il y a ${Math.floor(elapsed / MINUTE)} min`
	if (sameDay(date, now)) return `il y a ${Math.floor(elapsed / HOUR)} h`
	const yesterday = new Date(now)
	yesterday.setDate(now.getDate() - 1)
	if (sameDay(date, yesterday)) return `hier à ${TIME.format(date)}`
	return DAY.format(date)
}

export function formatDateTime(iso: string): string {
	return FULL.format(new Date(iso))
}

export function initials(name: string): string {
	return name
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((part) => part.charAt(0).toUpperCase())
		.join('')
}

export function joinNames(names: readonly string[]): string {
	if (names.length <= 1) return names[0] ?? ''
	return `${names.slice(0, -1).join(', ')} et ${names.at(-1)}`
}

export function memberHue(id: string): number {
	let hash = 0
	for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) % 360
	return hash
}
