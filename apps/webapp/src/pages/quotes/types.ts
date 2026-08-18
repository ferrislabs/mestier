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

export function eurosToCents(value: string): number {
	const normalized = value.replace(',', '.').trim()
	if (!normalized) return 0
	return Math.round(Number(normalized) * 100)
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

export function quoteLineTotalCents(line: QuoteLineFormValues): number {
	const quantity = Number(line.quantity.replace(',', '.'))
	if (!Number.isFinite(quantity) || quantity <= 0) return 0
	return Math.round(quantity * eurosToCents(line.unitPrice))
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
