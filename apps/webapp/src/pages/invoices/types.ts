import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'

// Re-export shared helpers from quotes rather than duplicating them
export {
	centsToEuros,
	customerContextDisplayName,
	customerDisplayName,
	eurosToCents,
	formatCents,
	formatDate,
	formatUnit,
	serviceRateDisplayName,
} from '#/pages/quotes/types'

export interface InvoiceLineFormValues {
	clientId: string
	catalogItemId: string
	catalogItemType: 'SERVICE' | 'PRODUCT' | 'CUSTOM'
	serviceRateId: string
	label: string
	quantity: string
	unit: ServiceRateUnit
	unitPrice: string
	vatRate: string
	notes: string
	photoKeys: string[]
}

export interface InvoiceFormValues {
	title: string
	customerId: string
	customerContextId: string
	emitterContextId: string
	legalMentionTemplateIds: string[]
	lines: InvoiceLineFormValues[]
}

export function emptyInvoiceLine(clientId = 'line-1'): InvoiceLineFormValues {
	return {
		clientId,
		catalogItemId: '',
		catalogItemType: 'CUSTOM',
		serviceRateId: '',
		label: '',
		quantity: '1',
		unit: 'HOUR',
		unitPrice: '',
		vatRate: '20',
		notes: '',
		photoKeys: [],
	}
}

export function invoiceStatusLabel(status: string): string {
	if (status === 'DRAFT') return 'Brouillon'
	if (status === 'SENT') return 'Envoyée'
	if (status === 'PARTIALLY_PAID') return 'Partiellement payée'
	if (status === 'PAID') return 'Payée'
	if (status === 'CANCELLED') return 'Annulée'
	return status
}
