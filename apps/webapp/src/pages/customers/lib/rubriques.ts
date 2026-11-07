import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { AccountKind } from '#/pages/customers/feature/mocks/account-extras'
import { formatMoney } from '#/pages/customers/lib/format'

export type ClientTab = 'identite' | 'contacts' | 'adresses' | 'activite'

export const TAB_INFO: Record<
	ClientTab,
	{ label: string; description: string }
> = {
	identite: {
		label: 'Identité',
		description: 'Coordonnées légales, suivi commercial et identifiant Stripe.',
	},
	contacts: {
		label: 'Contacts',
		description: 'Interlocuteurs du client.',
	},
	adresses: {
		label: 'Adresses',
		description: 'Facturation, chantiers et autres lieux.',
	},
	activite: {
		label: 'Activité',
		description: 'Opportunités, devis, factures, avoirs et paiements.',
	},
}

const TAB_ORDER: readonly ClientTab[] = [
	'identite',
	'contacts',
	'adresses',
	'activite',
]

export const CLIENT_TABS: readonly {
	id: ClientTab
	label: string
	description: string
}[] = TAB_ORDER.map((id) => ({ id, ...TAB_INFO[id] }))

export const DEFAULT_TAB: ClientTab = 'identite'

export function isClientTab(value: unknown): value is ClientTab {
	return CLIENT_TABS.some((tab) => tab.id === value)
}

export function validateClientSearch(raw: Record<string, unknown>): {
	onglet?: ClientTab
} {
	return isClientTab(raw.onglet) ? { onglet: raw.onglet } : {}
}

export function formatSiret(registrationNumber: string): string {
	const digits = registrationNumber.replace(/\s/g, '')
	if (!/^(\d{14}|\d{9})$/.test(digits)) return registrationNumber
	return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6, 9)} ${digits.slice(9)}`.trim()
}

export function identitySummary(
	customer: Pick<Customer, 'registration_number'>,
	kind: AccountKind,
	contexts: readonly Pick<CustomerContext, 'city'>[],
): string {
	const registration = (customer.registration_number ?? '').trim()
	const identity =
		kind === 'particulier'
			? 'Particulier'
			: registration === ''
				? 'SIRET non renseigné'
				: `SIRET ${formatSiret(registration)}`
	const city = contexts.find((context) => context.city?.trim())?.city?.trim()
	return city ? `${identity} · ${city}` : identity
}

export function contactsSummary(count: number): string {
	if (count === 0) return 'Aucun contact'
	return count === 1 ? '1 contact' : `${count} contacts`
}

export function addressesSummary(count: number): string {
	if (count === 0) return 'Aucune adresse'
	return count === 1 ? '1 adresse' : `${count} adresses`
}

export function activitySummary(outstanding: number): string {
	return outstanding > 0
		? `Encours ${formatMoney(outstanding)}`
		: 'Aucun encours'
}
