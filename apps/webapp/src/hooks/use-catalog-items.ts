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
	vatRate: string
	description: string
	customFields: { key: string; value: string }[]
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
}

export function useCatalogItems(
	serviceRates: ServiceRate[],
	products: Product[],
): CatalogItem[] {
	return useMemo(
		() => buildCatalogItems(serviceRates, products),
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
		})),
	]
}
