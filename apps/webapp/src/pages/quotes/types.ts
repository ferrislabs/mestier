import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { ServiceRate } from '#/hooks/use-reference-catalog'
import { formatUnit } from '#/lib/units'
import type { BillingFrequency } from '#/pages/quotes/lib/billing-frequency'

export { formatUnit, UNIT_GROUPS } from '#/lib/units'

export interface QuoteLineFormValues {
	clientId: string
	catalogItemId: string
	catalogItemType: 'SERVICE' | 'PRODUCT' | 'CUSTOM'
	serviceRateId: string
	label: string
	quantity: string
	/** One of `ServiceRateUnit`'s built-in codes, or an organization's own
	 * custom unit code (#449) — a plain string because a fixed union cannot
	 * name a value it does not know about yet. */
	unit: string
	unitPrice: string
	/** Basis points as a string for the input, `''` when unset. Only ever read
	 * by the server when the organization is subject to VAT — an organization
	 * that is not never sends this field, see `vatEnabled`. */
	vatRateBp: string
	notes: string
	photoKeys: string[]
	discountPercent: string
	frequency: BillingFrequency
}

/** The rates offered as one-click choices before an artisan types a custom
 * one — the standard French VAT schedule, most-charged first. */
export const COMMON_VAT_RATES_BP = [2000, 1000, 550, 210, 0] as const

export function formatVatRateBp(rateBp: number): string {
	const value = rateBp / 100
	return `${Number.isInteger(value) ? value : value.toFixed(2)} %`
}

/** `null` reads as "not yet sent", not as "the field is missing" — the
 * number is allocated only once the quote leaves Draft (#313). */
export function quoteReferenceLabel(
	reference: string | null | undefined,
): string {
	return reference ?? 'Numéro attribué à l’envoi'
}

export interface QuoteFormValues {
	title: string
	customerId: string
	customerContextId: string
	lines: QuoteLineFormValues[]
	ownerId: string | null
	validUntil: string
	depositPercent: string
	notes: string
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
		vatRateBp: '',
		notes: '',
		photoKeys: [],
		discountPercent: '',
		frequency: 'one_shot',
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

function scaleTo(digits: bigint, scale: number, targetScale: number): bigint {
	if (scale <= targetScale) {
		return digits * 10n ** BigInt(targetScale - scale)
	}

	const divisor = 10n ** BigInt(scale - targetScale)
	const quotient = digits / divisor
	const doubled = (digits % divisor) * 2n

	return doubled >= divisor ? quotient + 1n : quotient
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
	if (status === 'SUPERSEDED') return 'Remplacé'
	return status
}

export function quoteLineTotalCents(line: QuoteLineFormValues): number {
	const quantity = parseDecimal(line.quantity)
	if (!quantity || quantity.digits === 0n) return 0

	const unitPriceCents = BigInt(eurosToCents(line.unitPrice))
	const discount = parseDecimal(line.discountPercent) ?? {
		digits: 0n,
		scale: 0,
	}
	const remaining = 100n * 10n ** BigInt(discount.scale) - discount.digits
	if (remaining < 0n) return 0

	return Number(
		scaleTo(
			quantity.digits * unitPriceCents * remaining,
			quantity.scale + discount.scale + 2,
			0,
		),
	)
}

export interface QuoteLineDraftVatBreakdown {
	rateBp: number
	vatCents: number
}

export function quoteLinesVatBreakdown(
	lines: QuoteLineFormValues[],
	vatEnabled: boolean,
): QuoteLineDraftVatBreakdown[] {
	if (!vatEnabled) return []

	const byRate = new Map<number, bigint>()
	for (const line of lines) {
		const lineNet = BigInt(quoteLineTotalCents(line))
		if (lineNet === 0n) continue

		const rateBp = line.vatRateBp === '' ? 0 : Number(line.vatRateBp)
		byRate.set(rateBp, (byRate.get(rateBp) ?? 0n) + lineNet)
	}

	return Array.from(byRate.entries())
		.sort(([a], [b]) => a - b)
		.map(([rateBp, base]) => ({
			rateBp,
			vatCents: Number(scaleTo(base * BigInt(rateBp), 4, 0)),
		}))
}

export function quoteLinesGrossCents(
	netCents: number,
	vatBreakdown: QuoteLineDraftVatBreakdown[],
): number {
	return netCents + vatBreakdown.reduce((sum, line) => sum + line.vatCents, 0)
}

export function quoteLineSourceLabel(
	type: QuoteLineFormValues['catalogItemType'],
): string {
	if (type === 'SERVICE') return 'Service catalogue'
	if (type === 'PRODUCT') return 'Produit catalogue'
	return 'Ligne libre'
}
