// MOCK, pas d'endpoint, remplacé au lot B2 : type, TVA, site web, responsable, source, délai de paiement, Stripe, notes, dates de conversion et d'archivage, mobile des contacts, contact et adresse de facturation, type et pays des adresses.
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

export type AddressKind = 'billing' | 'site' | 'other'

export interface AccountExtras {
	kind: AccountKind | null
	vatNumber: string
	website: string
	ownerId: string | null
	source: LeadSource | null
	paymentTermsDays: number | null
	stripeCustomerId: string | null
	notes: string
	convertedAt: string | null
	archivedAt: string | null
	billingContactId: string | null
	billingContextId: string | null
	mobiles: Readonly<Record<string, string>>
	addressKinds: Readonly<Record<string, AddressKind>>
	addressCountries: Readonly<Record<string, string>>
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
	convertedAt: null,
	archivedAt: null,
	billingContactId: null,
	billingContextId: null,
	mobiles: {},
	addressKinds: {},
	addressCountries: {},
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
