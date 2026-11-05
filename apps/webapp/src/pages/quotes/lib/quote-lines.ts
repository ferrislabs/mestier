import type { CatalogItem } from '#/hooks/use-catalog-items'
import {
	centsToEuros,
	emptyQuoteLine,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'

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
