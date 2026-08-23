import { useMemo } from 'react'
import type {
	Product,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'

export type CatalogItemType = 'SERVICE' | 'PRODUCT'

export interface ProductCatalogFormValues {
	name: string
	sku: string
	unit: ServiceRateUnit
	unitPrice: string
	description: string
}

export interface CatalogItem {
	id: string
	type: CatalogItemType
	sourceId: string
	label: string
	unit: ServiceRateUnit
	unitPriceCents: number
	description: string
	sku?: string
	/** Prefills a quote line's VAT rate when this item is picked — never
	 * re-read afterwards, so editing it later does not rewrite a line
	 * already built from it. `null` when the referential has none set. */
	defaultVatRateBp: number | null
}

/**
 * Accepts the raw query data, `undefined` included, and defaults inside the
 * memo.
 *
 * Callers used to write `data?.data ?? []`, which builds a fresh array on every
 * render until the query resolves. That invalidated this memo each time, and the
 * quote edit screen has an effect keyed on the result that calls `setValues`:
 * a new identity per render made it an infinite render loop, which surfaced as
 * "Maximum update depth exceeded" whenever the catalogue was slow or failing.
 */
export function useCatalogItems(
	serviceRates: ServiceRate[] | undefined,
	products: Product[] | undefined,
): CatalogItem[] {
	return useMemo(
		() => buildCatalogItems(serviceRates ?? [], products ?? []),
		[products, serviceRates],
	)
}

export function buildCatalogItems(
	serviceRates: ServiceRate[],
	products: Product[],
): CatalogItem[] {
	return [
		...serviceRates.map((serviceRate) => ({
			id: `service:${serviceRate.id}`,
			type: 'SERVICE' as const,
			sourceId: serviceRate.id,
			label: serviceRate.label,
			unit: serviceRate.unit,
			unitPriceCents: serviceRate.rate_cents,
			description: '',
			defaultVatRateBp: serviceRate.default_vat_rate_bp ?? null,
		})),
		...products.map((product) => ({
			id: `product:${product.id}`,
			type: 'PRODUCT' as const,
			sourceId: product.id,
			label: product.name,
			unit: product.unit,
			unitPriceCents: product.unit_price_cents,
			description: product.description ?? '',
			sku: product.sku ?? '',
			defaultVatRateBp: product.default_vat_rate_bp ?? null,
		})),
	]
}
