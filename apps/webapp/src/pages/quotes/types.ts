import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type {
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'
import { formatUnit } from '#/lib/units'

export { formatUnit, UNIT_GROUPS } from '#/lib/units'

export interface QuoteLineFormValues {
	clientId: string
	catalogItemId: string
	catalogItemType: 'SERVICE' | 'PRODUCT' | 'CUSTOM'
	serviceRateId: string
	label: string
	quantity: string
	unit: ServiceRateUnit
	unitPrice: string
	notes: string
	photoKeys: string[]
}

export interface QuoteFormValues {
	title: string
	customerId: string
	customerContextId: string
	lines: QuoteLineFormValues[]
}

export function emptyQuoteLine(clientId = 'line-1'): QuoteLineFormValues {
	return {
		clientId,
		catalogItemId: '',
		catalogItemType: 'CUSTOM',
		serviceRateId: '',
		label: '',
		quantity: '1',
		unit: 'HOUR',
		unitPrice: '',
		notes: '',
		photoKeys: [],
	}
}

export function customerDisplayName(customer: Customer): string {
	return customer.name.trim()
}

export function customerContextDisplayName(
	customerContext: CustomerContext,
): string {
	const detail = [customerContext.postal_code, customerContext.city]
		.filter(Boolean)
		.join(' ')
	return detail ? `${customerContext.label} · ${detail}` : customerContext.label
}

/**
 * The address as it would be written on an envelope, one element per line.
 *
 * A quote is billed to a place, so the form shows the street and the town
 * rather than only the label an artisan gave the site. Empty when the address
 * was never filled in, which is what tells the form to say so out loud instead
 * of rendering a convincing blank.
 */
export function billingAddressLines(
	customerContext: CustomerContext,
): string[] {
	const town = [customerContext.postal_code, customerContext.city]
		.filter(Boolean)
		.join(' ')
	return [customerContext.address_line, town]
		.map((part) => part?.trim())
		.filter((part): part is string => Boolean(part))
}

export function serviceRateDisplayName(serviceRate: ServiceRate): string {
	return `${serviceRate.label} · ${formatUnit(serviceRate.unit)}`
}

export function centsToEuros(cents: number): string {
	return (cents / 100).toFixed(2)
}

/**
 * A decimal amount typed by hand, in exact cents.
 *
 * Integer arithmetic rather than `Number(value) * 100`, which is inexact: in
 * floating point `1.005 * 100` is `100.49999999999999`. A third decimal on a
 * euro amount has no authority to match, since the backend receives cents
 * already; it rounds the same way as everything else here, to even, so the file
 * has one rule rather than two.
 */
export function eurosToCents(value: string): number {
	const parsed = parseDecimal(value)
	if (!parsed) return 0

	return Number(scaleTo(parsed.digits, parsed.scale, 2))
}

/**
 * A hand-typed decimal, split into digits and scale: `"12,50"` becomes
 * `{ digits: 1250n, scale: 2 }`. `null` when there is nothing usable to read.
 *
 * Only non-negative values: a quantity must be positive and a price cannot be
 * negative, both enforced by the domain, so a sign here would be a bug
 * upstream rather than something to interpret.
 */
function parseDecimal(value: string): { digits: bigint; scale: number } | null {
	const normalized = value.replace(',', '.').trim()
	if (!/^\d*\.?\d*$/.test(normalized) || normalized === '.' || !normalized) {
		return null
	}

	const [whole = '', fraction = ''] = normalized.split('.')
	const digits = `${whole}${fraction}`
	if (!digits) return null

	return { digits: BigInt(digits), scale: fraction.length }
}

/**
 * Restates `digits × 10^-scale` at `targetScale`, rounding halves to even.
 *
 * Banker's rounding, because that is what `rust_decimal`'s `round_dp` does by
 * default and the backend computes the stored total with it. Verified rather
 * than assumed: on the backend `0.5` rounds to `0`, `1.5` to `2` and `2.5` to
 * `2`. `Math.round` rounds halves up instead, which is what made the preview
 * and the saved quote disagree by a cent about once every fifty lines.
 */
function scaleTo(digits: bigint, scale: number, targetScale: number): bigint {
	if (scale <= targetScale) {
		return digits * 10n ** BigInt(targetScale - scale)
	}

	const divisor = 10n ** BigInt(scale - targetScale)
	const quotient = digits / divisor
	const doubled = (digits % divisor) * 2n

	if (doubled > divisor) return quotient + 1n
	if (doubled < divisor) return quotient
	// Exactly a half: land on the even neighbour.
	return quotient % 2n === 0n ? quotient : quotient + 1n
}

export function formatCents(cents: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(cents / 100)
}

export function formatDate(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: 'short',
		year: 'numeric',
	}).format(new Date(value))
}

export function quoteStatusLabel(status: string): string {
	if (status === 'DRAFT') return 'Brouillon'
	if (status === 'SENT') return 'Envoyé'
	if (status === 'ACCEPTED') return 'Accepté'
	if (status === 'DECLINED') return 'Refusé'
	if (status === 'CANCELLED') return 'Annulé'
	return status
}

/**
 * The line's total, to the cent, computed the way the backend computes it.
 *
 * The stored total has always come from `domain::quote::service`, which
 * multiplies with `Decimal` and rounds halves to even. This preview used to
 * multiply with floats and round halves up, and the two disagreed by a cent
 * about once every fifty lines. Impossible to explain to a customer when it
 * lands, so this does the same integer arithmetic instead. `types.test.ts` pins
 * cases that used to diverge, and `domain::quote::service` pins the same ones.
 */
export function quoteLineTotalCents(line: QuoteLineFormValues): number {
	const quantity = parseDecimal(line.quantity)
	if (!quantity || quantity.digits === 0n) return 0

	const unitPriceCents = BigInt(eurosToCents(line.unitPrice))

	return Number(scaleTo(quantity.digits * unitPriceCents, quantity.scale, 0))
}

export function quoteLineSourceLabel(
	type: QuoteLineFormValues['catalogItemType'],
): string {
	if (type === 'SERVICE') return 'Service catalogue'
	if (type === 'PRODUCT') return 'Produit catalogue'
	return 'Ligne libre'
}

/**
 * One line summarising a folded quote line, in the order it is read aloud:
 * how much, of what, at what price.
 */
export function quoteLineSummary(line: QuoteLineFormValues): string {
	const quantity = line.quantity.trim() || '0'
	return `${quantity} ${formatUnit(line.unit)} × ${formatCents(
		eurosToCents(line.unitPrice),
	)}`
}
