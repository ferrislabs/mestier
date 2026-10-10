import type { DoeSupply } from '#/pages/doe/types'

export function pickedProductIds(supplies: DoeSupply[]): Set<string> {
	const ids = new Set<string>()
	for (const supply of supplies) {
		if (supply.source_product_id) ids.add(supply.source_product_id)
	}
	return ids
}

export function removeSuppliesBySource(
	supplies: DoeSupply[],
	productId: string,
): DoeSupply[] {
	return supplies.filter((supply) => supply.source_product_id !== productId)
}

export async function toggleProduct(
	supplies: DoeSupply[],
	productId: string,
	checked: boolean,
	fetchDraft: (productId: string) => Promise<DoeSupply>,
): Promise<DoeSupply[]> {
	if (!checked) return removeSuppliesBySource(supplies, productId)
	if (pickedProductIds(supplies).has(productId)) return supplies
	const draft = await fetchDraft(productId)
	return [...supplies, draft]
}
