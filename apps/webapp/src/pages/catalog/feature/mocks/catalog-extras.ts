// MOCK: catalog extras — no backend fields yet. Replace with category, cost, billing frequency and archived on products and service rates.
import type { BillingFrequency } from '#/pages/catalog/catalog-model'
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'

export interface CatalogExtras {
	category: string
	cost: number | null
	frequency: BillingFrequency
	archived: boolean
}

export const EMPTY_CATALOG_EXTRAS: CatalogExtras = {
	category: '',
	cost: null,
	frequency: 'one_shot',
	archived: false,
}

const store = createMockStore<Readonly<Record<string, CatalogExtras>>>({})

export function useCatalogExtrasMap(): Readonly<Record<string, CatalogExtras>> {
	return store.use()
}

export function useSaveCatalogExtras() {
	return {
		mutateAsync: async (itemId: string, patch: Partial<CatalogExtras>) => {
			await resolveAfterDelay(null)
			store.set((current) => ({
				...current,
				[itemId]: { ...(current[itemId] ?? EMPTY_CATALOG_EXTRAS), ...patch },
			}))
		},
	}
}
