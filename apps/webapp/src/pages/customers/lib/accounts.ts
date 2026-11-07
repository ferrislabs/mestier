import type { CustomerContact, CustomerContext } from '#/hooks/use-customers'
import {
	type AccountExtras,
	type AddressKind,
	EMPTY_EXTRAS,
} from '#/pages/customers/feature/mocks/account-extras'

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
>(contacts: readonly T[], extras: Pick<AccountExtras, 'billingContactId'>) {
	return (
		contacts.find((contact) => contact.id === extras.billingContactId) ??
		mainContactOf(contacts)
	)
}

export function contextKind(
	contextId: string,
	extras: Pick<AccountExtras, 'addressKinds'>,
): AddressKind {
	return extras.addressKinds[contextId] ?? 'site'
}

export function contextCountry(
	contextId: string,
	extras: Pick<AccountExtras, 'addressCountries'>,
): string {
	return extras.addressCountries[contextId] ?? 'France'
}

export function billingContextOf<T extends Pick<CustomerContext, 'id'>>(
	contexts: readonly T[],
	extras: Pick<
		AccountExtras,
		'billingContextId' | 'addressKinds' | 'addressCountries'
	>,
): T | undefined {
	return (
		contexts.find((context) => context.id === extras.billingContextId) ??
		contexts.find((context) => extras.addressKinds[context.id] === 'billing') ??
		contexts[0]
	)
}

export function directoryTerms(
	contacts: readonly Pick<
		CustomerContact,
		'id' | 'first_name' | 'last_name' | 'role' | 'email' | 'phone'
	>[],
	contexts: readonly Pick<CustomerContext, 'city'>[],
	extras: Pick<AccountExtras, 'mobiles'> = EMPTY_EXTRAS,
): SearchTerms {
	return {
		text: [
			...contexts.map((context) => context.city ?? ''),
			...contacts.flatMap((contact) => [
				contactName(contact),
				contact.email ?? '',
				contact.role ?? '',
			]),
		].filter(Boolean),
		phones: contacts
			.flatMap((contact) => [contact.phone ?? '', extras.mobiles[contact.id]])
			.filter((phone): phone is string => Boolean(phone)),
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

export function withEntry<V extends string>(
	record: Readonly<Record<string, V>>,
	key: string,
	value: V | '',
): Record<string, V> {
	const { [key]: _removed, ...rest } = record
	return value === '' ? rest : { ...rest, [key]: value }
}
