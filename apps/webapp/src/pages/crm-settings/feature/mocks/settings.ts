// MOCK: CRM settings — no backend endpoint yet. Replace with an organization billing-settings resource (company legal mentions, payment terms, numbering sequences, Stripe webhook secret status).

import {
	cleanCompany,
	raiseSequence,
} from '#/pages/crm-settings/lib/settings-model'
import type {
	Company,
	CrmSettingsState,
	SequenceKind,
	Settings,
} from '#/pages/crm-settings/types'
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'

const COMPANY: Company = {
	name: 'Menuiserie Duval',
	legalForm: 'SARL',
	capital: '50 000 €',
	legalId: '82345678900019',
	registry: 'RCS Nantes B 823 456 789',
	vatNumber: 'FR12823456789',
	address: {
		street: '14 rue des Artisans',
		complement: '',
		postalCode: '44800',
		city: 'Saint-Herblain',
		country: 'France',
	},
	email: 'contact@menuiserie-duval.fr',
	phone: '02 40 12 34 56',
	iban: 'FR76 3000 6000 0112 3456 7890 189',
	bic: 'AGRIFRPP',
	insurance:
		'Assurance décennale AXA France n° 12345678, couverture France métropolitaine.',
}

const INITIAL: CrmSettingsState = {
	settings: {
		company: COMPANY,
		paymentTermsDays: 30,
		quoteValidityDays: 30,
		defaultVatRate: 20,
		latePenalty: 'trois fois le taux d’intérêt légal',
		recoveryFee: 4000,
		invoiceFooter: 'Merci de votre confiance.',
		numberPadding: 3,
		stripeEndpoint: 'https://api.mestier.app/webhooks/stripe/menuiserie-duval',
		stripeSecretSet: true,
	},
	sequences: {
		quote: { [String(new Date().getFullYear())]: 210 },
		invoice: { [String(new Date().getFullYear())]: 112 },
		creditNote: { [String(new Date().getFullYear())]: 3 },
	},
}

export const crmSettingsStore = createMockStore<CrmSettingsState>(INITIAL)

export function useCrmSettings() {
	const data = crmSettingsStore.use()
	return { data, isLoading: false }
}

export function readCrmSettings(): Settings {
	return crmSettingsStore.get().settings
}

export function useUpdateCompany() {
	return {
		isPending: false,
		mutateAsync: async (company: Company) => {
			await resolveAfterDelay(null)
			crmSettingsStore.set((current) => ({
				...current,
				settings: { ...current.settings, company: cleanCompany(company) },
			}))
		},
	}
}

export function useUpdateBilling() {
	return {
		isPending: false,
		mutateAsync: async (
			patch: Partial<Omit<Settings, 'company' | 'stripeEndpoint'>>,
		) => {
			await resolveAfterDelay(null)
			crmSettingsStore.set((current) => ({
				...current,
				settings: { ...current.settings, ...patch },
			}))
		},
	}
}

export function useReplaceStripeSecret() {
	return {
		isPending: false,
		mutateAsync: async () => {
			await resolveAfterDelay(null)
			crmSettingsStore.set((current) => ({
				...current,
				settings: { ...current.settings, stripeSecretSet: true },
			}))
		},
	}
}

export function useRaiseSequences() {
	return {
		isPending: false,
		mutateAsync: async (
			year: number,
			raises: readonly { kind: SequenceKind; last: number }[],
		) => {
			await resolveAfterDelay(null)
			crmSettingsStore.set((current) => ({
				...current,
				sequences: raises.reduce(
					(sequences, raise) =>
						raiseSequence(sequences, raise.kind, year, raise.last),
					current.sequences,
				),
			}))
		},
	}
}
