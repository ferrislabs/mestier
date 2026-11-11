export type Parsed<T> =
	| { kind: 'empty' }
	| { kind: 'value'; value: T }
	| { kind: 'invalid' }

const EMPTY = { kind: 'empty' } as const
const INVALID = { kind: 'invalid' } as const
const SPACES = /[\s  ]/g
const DECIMAL = /^-?(\d+(\.\d*)?|\.\d+)$/
const ISO_DATE = /^(\d{4})-(\d{2})-(\d{2})$/
const FRENCH_DATE = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/

function normalizeDecimal(input: string): string {
	const compact = input.replace(SPACES, '')
	if (compact.includes(',') && compact.includes('.')) {
		return compact.replace(/\./g, '').replace(',', '.')
	}
	return compact.replace(',', '.')
}

export function parseNumberInput(input: string): Parsed<number> {
	if (input.trim() === '') return EMPTY
	const normalized = normalizeDecimal(input)
	if (!DECIMAL.test(normalized)) return INVALID
	const value = Number(normalized)
	return Number.isFinite(value) ? { kind: 'value', value } : INVALID
}

export function parseEurosInput(input: string): Parsed<number> {
	const withoutSign = input.replace('€', '')
	if (withoutSign.trim() === '') return EMPTY
	const normalized = normalizeDecimal(withoutSign)
	if (!DECIMAL.test(normalized)) return INVALID
	const decimals = normalized.split('.')[1] ?? ''
	if (decimals.length > 2) return INVALID
	const cents = Math.round(Number(normalized) * 100)
	return Number.isSafeInteger(cents) ? { kind: 'value', value: cents } : INVALID
}

export function formatNumber(
	value: number,
	decimals: number | null | undefined,
): string {
	const fixed = decimals ?? null
	return new Intl.NumberFormat('fr-FR', {
		minimumFractionDigits: fixed ?? 0,
		maximumFractionDigits: fixed ?? 4,
	}).format(value)
}

export function formatCents(cents: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(cents / 100)
}

export function numberInputText(value: number): string {
	return String(value).replace('.', ',')
}

export function centsInputText(cents: number): string {
	const euros = cents / 100
	return Number.isInteger(euros)
		? String(euros)
		: euros.toFixed(2).replace('.', ',')
}

function isRealDate(year: number, month: number, day: number): boolean {
	const date = new Date(Date.UTC(year, month - 1, day))
	return (
		date.getUTCFullYear() === year &&
		date.getUTCMonth() === month - 1 &&
		date.getUTCDate() === day
	)
}

function isoOf(year: number, month: number, day: number): string {
	return `${String(year).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
}

export function parseDateInput(input: string): Parsed<string> {
	const text = input.trim()
	if (text === '') return EMPTY
	const iso = ISO_DATE.exec(text)
	const french = FRENCH_DATE.exec(text)
	const parts = iso
		? [Number(iso[1]), Number(iso[2]), Number(iso[3])]
		: french
			? [Number(french[3]), Number(french[2]), Number(french[1])]
			: null
	if (!parts) return INVALID
	const [year, month, day] = parts
	if (!isRealDate(year, month, day)) return INVALID
	return { kind: 'value', value: isoOf(year, month, day) }
}

export function formatIsoDate(value: string): string {
	const match = ISO_DATE.exec(value)
	if (!match) return value
	return `${match[3]}/${match[2]}/${match[1]}`
}
