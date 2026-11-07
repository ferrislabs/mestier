import type {
	Customer,
	CustomerContact,
	CustomerContext,
	CustomerListItem,
	LeadSource,
} from '#/hooks/use-customers'
import {
	type AccountKind,
	type AddressKind,
	addressKindOf,
	kindOf,
} from '#/pages/customers/types'

type NamedContact = Pick<CustomerContact, 'first_name' | 'last_name'>

export type SearchTerms = {
	text: readonly string[]
	phones: readonly string[]
}

export const NO_TERMS: SearchTerms = { text: [], phones: [] }

export function contactName(contact: NamedContact): string {
	return `${contact.first_name.trim()} ${contact.last_name.trim()}`.trim()
}

export function mainContactOf<T extends Pick<CustomerContact, 'is_primary'>>(
	contacts: readonly T[],
): T | undefined {
	return contacts.find((contact) => contact.is_primary) ?? contacts[0]
}

export function billingContactOf<
	T extends Pick<CustomerContact, 'id' | 'is_primary'>,
>(contacts: readonly T[], billingContactId: string | null | undefined) {
	return (
		contacts.find((contact) => contact.id === billingContactId) ??
		mainContactOf(contacts)
	)
}

export function contextKind(
	context: Pick<CustomerContext, 'kind'>,
): AddressKind {
	return addressKindOf(context.kind)
}

const DEFAULT_COUNTRY_CODE = 'FR'

const DEFAULT_COUNTRY_LABEL = 'France'

export function countryLabelOf(country: string): string {
	const trimmed = country.trim()
	return trimmed === '' || trimmed.toUpperCase() === DEFAULT_COUNTRY_CODE
		? DEFAULT_COUNTRY_LABEL
		: trimmed
}

export function countryCodeOf(input: string): string {
	const trimmed = input.trim()
	return trimmed === '' ||
		trimmed.toLowerCase() === DEFAULT_COUNTRY_LABEL.toLowerCase()
		? DEFAULT_COUNTRY_CODE
		: trimmed
}

export function billingContextOf<
	T extends Pick<CustomerContext, 'id' | 'kind'>,
>(
	contexts: readonly T[],
	billingAddressId: string | null | undefined,
): T | undefined {
	return (
		contexts.find((context) => context.id === billingAddressId) ??
		contexts.find((context) => context.kind === 'BILLING') ??
		contexts[0]
	)
}

export interface AccountDetails {
	kind: AccountKind
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
	billingAddressId: string | null
}

export function accountDetailsOf(
	customer: Customer,
	stripeCustomerId: string | null,
): AccountDetails {
	return {
		kind: kindOf(customer),
		vatNumber: customer.vat_number ?? '',
		website: customer.website ?? '',
		ownerId: customer.owner_member_id ?? null,
		source: customer.lead_source ?? null,
		paymentTermsDays: customer.payment_terms_days ?? null,
		stripeCustomerId,
		notes: customer.notes ?? '',
		convertedAt: customer.converted_at ?? null,
		archivedAt: customer.archived_at ?? null,
		billingContactId: customer.billing_contact_id ?? null,
		billingAddressId: customer.billing_address_id ?? null,
	}
}

export function rowTerms(
	row: Pick<CustomerListItem, 'city' | 'primary_contact'>,
): SearchTerms {
	const contact = row.primary_contact
	return {
		text: [
			row.city ?? '',
			contact ? contactName(contact) : '',
			contact?.email ?? '',
			contact?.role ?? '',
		].filter(Boolean),
		phones: [contact?.phone ?? '', contact?.mobile ?? ''].filter(Boolean),
	}
}

export type CustomerListData = {
	city: string
	contactName: string
	contactRole: string
	outstanding: number
	overdue: number
	revenue12m: number
	lastActivity: string | null
}

export function openDealsSentence(count: number | null): string {
	if (count === null)
		return 'Ses opportunités ouvertes seront marquées comme perdues (motif : compte archivé) et ne pourront pas être rouvertes.'
	if (count === 0) return 'Aucune opportunité ouverte n’est concernée.'
	return count === 1
		? 'Son opportunité ouverte sera marquée comme perdue (motif : compte archivé) et ne pourra pas être rouverte.'
		: `Ses ${count} opportunités ouvertes seront marquées comme perdues (motif : compte archivé) et ne pourront pas être rouvertes.`
}
