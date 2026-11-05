import type { QuoteFormValues } from '#/pages/quotes/types'

export interface NewQuoteErrors {
	customerId?: string
	customerContextId?: string
	title?: string
	lines?: string
}

export const NEW_QUOTE_FIELD_IDS: Record<keyof NewQuoteErrors, string> = {
	customerId: 'new-quote-client',
	customerContextId: 'new-quote-address',
	title: 'new-quote-title',
	lines: 'new-quote-lines',
}

export function isValidQuoteLine(line: QuoteFormValues['lines'][number]) {
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

export function validateNewQuote(values: QuoteFormValues): NewQuoteErrors {
	return {
		...(values.customerId === '' ? { customerId: 'Choisis un client.' } : {}),
		...(values.customerId !== '' && values.customerContextId === ''
			? { customerContextId: 'Choisis une adresse de facturation.' }
			: {}),
		...(values.title.trim() === ''
			? { title: 'L’objet du devis est obligatoire.' }
			: {}),
		...(values.lines.length === 0 || !values.lines.every(isValidQuoteLine)
			? {
					lines:
						'Chaque ligne a besoin d’une désignation, d’une quantité positive et d’un prix.',
				}
			: {}),
	}
}

export function firstInvalidFieldId(errors: NewQuoteErrors): string | null {
	const first = (
		Object.keys(NEW_QUOTE_FIELD_IDS) as (keyof NewQuoteErrors)[]
	).find((key) => errors[key])
	return first ? NEW_QUOTE_FIELD_IDS[first] : null
}
