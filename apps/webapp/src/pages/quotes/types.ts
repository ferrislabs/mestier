import type { Customer, Property } from '#/hooks/use-customers'
import type {
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'

export interface QuoteLineFormValues {
	clientId: string
	serviceRateId: string
	label: string
	quantity: string
	unit: ServiceRateUnit
	unitPrice: string
	notes: string
	photoKeys: string[]
}

export interface QuoteFormValues {
	customerId: string
	propertyId: string
	lines: QuoteLineFormValues[]
}

export function emptyQuoteLine(clientId = 'line-1'): QuoteLineFormValues {
	return {
		clientId,
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
	return `${customer.first_name} ${customer.last_name}`.trim()
}

export function propertyDisplayName(property: Property): string {
	return `${property.label} · ${property.city}`.trim()
}

export function serviceRateDisplayName(serviceRate: ServiceRate): string {
	return `${serviceRate.label} · ${formatUnit(serviceRate.unit)}`
}

export function formatUnit(unit: ServiceRateUnit): string {
	if (unit === 'HOUR') return 'heure'
	if (unit === 'ML') return 'ml'
	return 'm2'
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
