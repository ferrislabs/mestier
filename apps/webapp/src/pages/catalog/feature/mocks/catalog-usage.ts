// MOCK, pas d'endpoint, remplacé au lot B6 : les lignes de devis et de factures ne portent pas encore l'article du catalogue.
import { useMemo } from 'react'
import { type CatalogUsage, catalogUsage } from '#/pages/catalog/catalog-model'
import { createMockStore } from '#/pages/customers/feature/mocks/mock-store'

interface UsageLines {
	quoteLines: readonly (string | null)[]
	invoiceLines: readonly (string | null)[]
}

const store = createMockStore<UsageLines>({ quoteLines: [], invoiceLines: [] })

export function useCatalogUsageMap(): ReadonlyMap<string, CatalogUsage> {
	const lines = store.use()
	return useMemo(() => catalogUsage(lines), [lines])
}
