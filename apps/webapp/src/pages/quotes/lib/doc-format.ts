const DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/

const DATE_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	year: 'numeric',
})

function toDate(value: string): Date {
	const match = DATE_ONLY.exec(value)
	if (match)
		return new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]))
	return new Date(value)
}

export function formatDocumentDate(value: string | null | undefined): string {
	if (!value) return '—'
	const date = toDate(value)
	return Number.isNaN(date.getTime()) ? '—' : DATE_FORMAT.format(date)
}

export function formatDocumentDateTime(value: string): string {
	const date = toDate(value)
	if (Number.isNaN(date.getTime())) return value
	const hours = String(date.getHours()).padStart(2, '0')
	const minutes = String(date.getMinutes()).padStart(2, '0')
	return `${DATE_FORMAT.format(date)}, ${hours}:${minutes}`
}

export function plural(
	count: number,
	singular: string,
	pluralForm: string,
): string {
	return Math.abs(count) < 2 ? singular : pluralForm
}

const DAY_MS = 86_400_000

function startOfDay(date: Date): number {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()
}

export function dueLabel(
	dueAt: string | null | undefined,
	now: Date = new Date(),
): string {
	if (!dueAt) return 'Sans échéance'
	const due = toDate(dueAt)
	if (Number.isNaN(due.getTime())) return 'Sans échéance'
	const late = Math.round((startOfDay(now) - startOfDay(due)) / DAY_MS)
	if (late === 0) return 'Échéance aujourd’hui'
	if (late > 0) return `Échue depuis ${late} ${plural(late, 'jour', 'jours')}`
	return `À régler dans ${-late} ${plural(-late, 'jour', 'jours')}`
}

export function formatSiret(legalId: string): string {
	const digits = legalId.replace(/\s/g, '')
	if (!/^\d{14}$/.test(digits)) return legalId
	return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6, 9)} ${digits.slice(9)}`
}

export function postalAddressLines(address: {
	street?: string | null
	complement?: string | null
	postalCode?: string | null
	city?: string | null
	country?: string | null
}): string[] {
	const town = [address.postalCode, address.city].filter(Boolean).join(' ')
	const country =
		address.country?.trim() === 'France' ? '' : (address.country?.trim() ?? '')
	return [address.street, address.complement, town, country]
		.map((line) => line?.trim() ?? '')
		.filter((line) => line !== '')
}

export function nameHue(name: string): number {
	let hash = 0
	for (const char of name) hash = (hash * 31 + char.charCodeAt(0)) % 360
	return hash
}

const PREFIXES =
	/^(M\.|Mme|Famille|SCI|École|Cabinet|Résidence)\s+(et\s+Mme\s+)?/i

export function initialsOf(name: string): string {
	return name
		.replace(PREFIXES, '')
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((word) => word.charAt(0).toUpperCase())
		.join('')
}

export function normalizeText(text: string): string {
	return text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim()
}
