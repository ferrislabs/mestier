import { newInvoiceTermsError } from '#/pages/invoices/lib/invoice-new'
import type {
	InvoiceFormValues,
	InvoiceLineFormValues,
} from '#/pages/invoices/types'

export interface NewInvoiceFormValues extends InvoiceFormValues {
	termsDays: string
}

export interface NewInvoiceErrors {
	customerId?: string
	customerContextId?: string
	terms?: string
	lines?: string
}

export const NEW_INVOICE_FIELD_IDS: Record<keyof NewInvoiceErrors, string> = {
	customerId: 'new-invoice-client',
	customerContextId: 'new-invoice-address',
	terms: 'new-invoice-terms',
	lines: 'new-invoice-lines',
}

export function isValidInvoiceLine(line: InvoiceLineFormValues): boolean {
	const quantity = Number(line.quantity.replace(',', '.'))
	const unitPrice = Number(line.unitPrice.replace(',', '.'))
	return (
		line.label.trim() !== '' &&
		Number.isFinite(quantity) &&
		quantity > 0 &&
		line.unitPrice.trim() !== '' &&
		Number.isFinite(unitPrice) &&
		unitPrice >= 0
	)
}

export function validateNewInvoice(
	values: NewInvoiceFormValues,
): NewInvoiceErrors {
	const terms = newInvoiceTermsError(values.termsDays)
	return {
		...(terms ? { terms } : {}),
		...(values.customerId === '' ? { customerId: 'Choisis un client.' } : {}),
		...(values.customerId !== '' && values.customerContextId === ''
			? { customerContextId: 'Choisis une adresse de facturation.' }
			: {}),
		...(values.lines.length === 0 || !values.lines.every(isValidInvoiceLine)
			? {
					lines:
						'Chaque ligne a besoin d’une désignation, d’une quantité positive et d’un prix.',
				}
			: {}),
	}
}

export function firstInvalidInvoiceFieldId(
	errors: NewInvoiceErrors,
): string | null {
	const first = (
		Object.keys(NEW_INVOICE_FIELD_IDS) as (keyof NewInvoiceErrors)[]
	).find((key) => errors[key])
	return first ? NEW_INVOICE_FIELD_IDS[first] : null
}
