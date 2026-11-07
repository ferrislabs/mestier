import type { CatalogItem } from '#/hooks/use-catalog-items'
import { lineTermsBody } from '#/pages/quotes/lib/quote-form-terms'
import {
	centsToEuros,
	emptyQuoteLine,
	eurosToCents,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'

const PRODUCT_PREFIX = 'product:'

export function productIdOf(line: QuoteLineFormValues): string | null {
	return line.catalogItemType === 'PRODUCT' &&
		line.catalogItemId.startsWith(PRODUCT_PREFIX)
		? line.catalogItemId.slice(PRODUCT_PREFIX.length)
		: null
}

export function quoteLineBodies(lines: QuoteLineFormValues[]) {
	return lines.map((line) => ({
		service_rate_id: line.serviceRateId || null,
		product_id: productIdOf(line),
		label: line.label.trim(),
		quantity: line.quantity.replace(',', '.').trim(),
		unit: line.unit,
		unit_price_cents: eurosToCents(line.unitPrice),
		vat_rate_bp: line.vatRateBp === '' ? null : Number(line.vatRateBp),
		...lineTermsBody(line),
		notes: line.notes.trim() || null,
		photo_keys: line.photoKeys,
	}))
}

export function lineFromCatalogItem(
	item: CatalogItem,
	clientId: string,
): QuoteLineFormValues {
	return {
		...emptyQuoteLine(clientId),
		catalogItemId: item.id,
		catalogItemType: item.type,
		serviceRateId: item.type === 'SERVICE' ? item.sourceId : '',
		label: item.label,
		unit: item.unit,
		unitPrice: centsToEuros(item.unitPriceCents),
		vatRateBp:
			item.defaultVatRateBp !== null ? String(item.defaultVatRateBp) : '',
		notes: item.description,
	}
}

export function copyQuoteLine(
	line: QuoteLineFormValues,
	clientId: string,
): QuoteLineFormValues {
	return { ...line, clientId, photoKeys: [...line.photoKeys] }
}
