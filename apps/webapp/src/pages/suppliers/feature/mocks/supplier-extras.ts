// MOCK, pas d'endpoint, remplacé au lot B6 : IBAN, contacts, adresses typées, catégorie par défaut, délai de paiement, notes et archivage des fournisseurs.
import { createMockStore } from '#/pages/customers/feature/mocks/mock-store'
import type { SupplierExtras } from '#/pages/suppliers/types'

export const EMPTY_SUPPLIER_EXTRAS: SupplierExtras = {
	iban: '',
	defaultCategoryId: null,
	paymentTermsDays: null,
	notes: '',
	archived: false,
	contacts: null,
	addresses: null,
}

const STORAGE_KEY = 'mestier.mock.supplier-extras'

type Extras = Readonly<Record<string, SupplierExtras>>

function load(): Extras {
	try {
		const raw = window.localStorage.getItem(STORAGE_KEY)
		return raw ? (JSON.parse(raw) as Extras) : {}
	} catch {
		return {}
	}
}

const store = createMockStore<Extras>(load())

function persist(next: Extras) {
	try {
		window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
	} catch {
		return
	}
}

export function useSupplierExtrasMap(): Extras {
	return store.use()
}

export function useSaveSupplierExtras() {
	return {
		mutate: (supplierId: string, patch: Partial<SupplierExtras>) => {
			store.set((current) => {
				const next = {
					...current,
					[supplierId]: {
						...(current[supplierId] ?? EMPTY_SUPPLIER_EXTRAS),
						...patch,
					},
				}
				persist(next)
				return next
			})
		},
	}
}
