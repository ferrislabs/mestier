const MINUTES_PER_HOUR = 60
const MINUTES_PER_DAY = 1440

export function formatAgo(minutes: number): string {
	if (minutes < 1) return 'à l’instant'
	if (minutes < MINUTES_PER_HOUR) return `il y a ${Math.round(minutes)} min`
	if (minutes < MINUTES_PER_DAY)
		return `il y a ${Math.round(minutes / MINUTES_PER_HOUR)} h`
	const days = Math.round(minutes / MINUTES_PER_DAY)
	return days === 1 ? 'il y a 1 jour' : `il y a ${days} jours`
}

export function formatDuration(seconds: number): string {
	return `${seconds.toLocaleString('fr-FR', { maximumFractionDigits: 1 })} s`
}

const MS_PER_SECOND = 1000
const MS_PER_MINUTE = 60_000
const MS_PER_HOUR = 3_600_000
const MS_PER_DAY = 86_400_000

export function formatMilliseconds(ms: number): string {
	if (ms < MS_PER_SECOND) return `${Math.round(ms)} ms`
	if (ms < MS_PER_MINUTE) return formatDuration(ms / MS_PER_SECOND)
	if (ms < MS_PER_HOUR) {
		const minutes = Math.floor(ms / MS_PER_MINUTE)
		const seconds = Math.round((ms % MS_PER_MINUTE) / MS_PER_SECOND)
		return seconds > 0 ? `${minutes} min ${seconds} s` : `${minutes} min`
	}
	if (ms < MS_PER_DAY) {
		const hours = Math.floor(ms / MS_PER_HOUR)
		const minutes = Math.round((ms % MS_PER_HOUR) / MS_PER_MINUTE)
		return minutes > 0 ? `${hours} h ${minutes} min` : `${hours} h`
	}
	return `${Math.round(ms / MS_PER_DAY)} j`
}

export function formatItemCount(count: number): string {
	return count > 1 ? `${count} éléments` : `${count} élément`
}

export function formatUsage(workflowCount: number): string {
	if (workflowCount === 0) return 'Non utilisée'
	return workflowCount === 1
		? 'Utilisée par 1 workflow'
		: `Utilisée par ${workflowCount} workflows`
}

export function minutesSince(iso: string, now: number = Date.now()): number {
	const elapsed = (now - new Date(iso).getTime()) / 60_000
	return Number.isFinite(elapsed) ? Math.max(0, elapsed) : 0
}

const MONTHS = [
	'janv.',
	'févr.',
	'mars',
	'avr.',
	'mai',
	'juin',
	'juil.',
	'août',
	'sept.',
	'oct.',
	'nov.',
	'déc.',
]

function pad(value: number): string {
	return String(value).padStart(2, '0')
}

export function formatStartedAt(iso: string, now: number = Date.now()): string {
	const date = new Date(iso)
	if (Number.isNaN(date.getTime())) return '—'
	const time = `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
	const today = new Date(now)
	today.setHours(0, 0, 0, 0)
	const startOfDay = new Date(date)
	startOfDay.setHours(0, 0, 0, 0)
	const dayDiff = Math.round(
		(today.getTime() - startOfDay.getTime()) / 86_400_000,
	)
	if (dayDiff === 0) return `Aujourd’hui à ${time}`
	if (dayDiff === 1) return `Hier à ${time}`
	return `${date.getDate()} ${MONTHS[date.getMonth()]} ${date.getFullYear()} à ${time}`
}

export function formatVersionLabel(iso: string): string {
	const date = new Date(iso)
	if (Number.isNaN(date.getTime())) return 'Version'
	return `Version du ${date.getDate()} ${MONTHS[date.getMonth()]} ${date.getFullYear()} à ${pad(date.getHours())}:${pad(date.getMinutes())}`
}
