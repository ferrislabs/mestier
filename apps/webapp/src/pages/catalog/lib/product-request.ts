import type { ProductCatalogFormValues } from '#/hooks/use-catalog-items'
import { eurosToCents } from '#/pages/quotes/types'

export const EMPTY_TECHNICAL_SHEET = {
	supplierId: '',
	manufacturer: '',
	technicalSpecs: '',
	datasheetKeys: [] as string[],
}

function optional(value: string): string | null {
	const trimmed = value.trim()
	return trimmed.length > 0 ? trimmed : null
}

export function buildProductRequest(values: ProductCatalogFormValues) {
	return {
		name: values.name.trim(),
		sku: optional(values.sku),
		unit: values.unit,
		unit_price_cents: eurosToCents(values.unitPrice),
		description: optional(values.description),
		photo_keys: values.photoKeys,
		supplier_id: values.supplierId.length > 0 ? values.supplierId : null,
		manufacturer: optional(values.manufacturer),
		technical_specs: optional(values.technicalSpecs),
		datasheet_keys: values.datasheetKeys,
	}
}

export function datasheetName(key: string): string {
	return key.split('/').at(-1) ?? key
}
