// MOCK: account extras — no backend fields yet. Replace with kind, vat_number, website, owner_id, source, payment_terms_days, stripe_customer_id and notes on the customer resource.
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'

export type AccountKind = 'particulier' | 'professionnel'

export type LeadSource =
	| 'bouche_a_oreille'
	| 'site_web'
	| 'recommandation'
	| 'salon'
	| 'architecte'
	| 'autre'

export interface AccountExtras {
	kind: AccountKind | null
	vatNumber: string
	website: string
	ownerId: string | null
	source: LeadSource | null
	paymentTermsDays: number | null
	stripeCustomerId: string | null
	notes: string
}

export const EMPTY_EXTRAS: AccountExtras = {
	kind: null,
	vatNumber: '',
	website: '',
	ownerId: null,
	source: null,
	paymentTermsDays: null,
	stripeCustomerId: null,
	notes: '',
}

const store = createMockStore<Readonly<Record<string, AccountExtras>>>({})

export function useAccountExtrasMap(): Readonly<Record<string, AccountExtras>> {
	return store.use()
}

export function useAccountExtras(customerId: string): AccountExtras {
	return store.use()[customerId] ?? EMPTY_EXTRAS
}

export function useSaveAccountExtras() {
	return {
		isPending: false,
		mutateAsync: async (customerId: string, patch: Partial<AccountExtras>) => {
			await resolveAfterDelay(null)
			store.set((current) => ({
				...current,
				[customerId]: {
					...(current[customerId] ?? EMPTY_EXTRAS),
					...patch,
				},
			}))
		},
	}
}
