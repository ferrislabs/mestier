import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import {
	accountDetailsOf,
	billingContactOf,
	billingContextOf,
	contactName,
	contextKind,
	countryCodeOf,
	countryLabelOf,
	directoryTerms,
	mainContactOf,
	openDealsSentence,
} from '#/pages/customers/lib/accounts'

const contacts = [
	{ id: 'k1', first_name: 'Léa', last_name: 'Duval', is_primary: false },
	{ id: 'k2', first_name: 'Paul', last_name: 'Roux', is_primary: true },
]

const contexts = [
	{ id: 'a1', label: 'Chantier', city: 'Rezé', kind: 'SITE' as const },
	{ id: 'a2', label: 'Siège', city: 'Nantes', kind: 'SITE' as const },
]

function customerFixture(overrides: Partial<Customer>): Customer {
	return {
		id: 'c1',
		organization_id: 'org',
		name: 'Boulangerie Lemoine',
		status: 'CLIENT',
		kind: 'PROFESSIONAL',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

describe('contacts', () => {
	it('names a contact', () => {
		expect(contactName({ first_name: ' Léa ', last_name: 'Duval' })).toBe(
			'Léa Duval',
		)
	})

	it('picks the primary contact, or the first one', () => {
		expect(mainContactOf(contacts)?.id).toBe('k2')
		expect(mainContactOf([contacts[0]])?.id).toBe('k1')
		expect(mainContactOf([])).toBeUndefined()
	})

	it('picks the designated billing contact, then the primary one', () => {
		expect(billingContactOf(contacts, 'k1')?.id).toBe('k1')
		expect(billingContactOf(contacts, 'gone')?.id).toBe('k2')
		expect(billingContactOf(contacts, null)?.id).toBe('k2')
		expect(billingContactOf(contacts, undefined)?.id).toBe('k2')
	})
})

describe('addresses', () => {
	it('prefers the designated billing address, then a billing-kind one, then the first', () => {
		expect(billingContextOf(contexts, 'a2')?.id).toBe('a2')
		expect(
			billingContextOf(
				[
					{ id: 'a1', kind: 'SITE' as const },
					{ id: 'a2', kind: 'BILLING' as const },
				],
				null,
			)?.id,
		).toBe('a2')
		expect(billingContextOf(contexts, null)?.id).toBe('a1')
		expect(billingContextOf([], null)).toBeUndefined()
	})

	it('maps the API kind to the screen vocabulary', () => {
		expect(contextKind({ kind: 'SITE' })).toBe('site')
		expect(contextKind({ kind: 'BILLING' })).toBe('billing')
		expect(contextKind({ kind: 'OTHER' })).toBe('other')
	})

	it('shows the stored country code as a country name', () => {
		expect(countryLabelOf('FR')).toBe('France')
		expect(countryLabelOf(' fr ')).toBe('France')
		expect(countryLabelOf('')).toBe('France')
		expect(countryLabelOf('BE')).toBe('BE')
		expect(countryLabelOf('Belgique')).toBe('Belgique')
	})

	it('stores France as its code and keeps any other country as typed', () => {
		expect(countryCodeOf('France')).toBe('FR')
		expect(countryCodeOf(' france ')).toBe('FR')
		expect(countryCodeOf('')).toBe('FR')
		expect(countryCodeOf('Belgique')).toBe('Belgique')
	})
})

describe('accountDetailsOf', () => {
	it('reads every detail from the customer and the Stripe mock', () => {
		const details = accountDetailsOf(
			customerFixture({
				kind: 'INDIVIDUAL',
				vat_number: 'FR123',
				website: 'https://x.fr',
				owner_member_id: 'm1',
				lead_source: 'salon',
				payment_terms_days: 45,
				notes: 'Livraison le matin',
				converted_at: '2026-10-01T09:00:00Z',
				archived_at: null,
				billing_contact_id: 'k1',
				billing_address_id: 'a2',
			}),
			'cus_123',
		)
		expect(details).toEqual({
			kind: 'particulier',
			vatNumber: 'FR123',
			website: 'https://x.fr',
			ownerId: 'm1',
			source: 'salon',
			paymentTermsDays: 45,
			stripeCustomerId: 'cus_123',
			notes: 'Livraison le matin',
			convertedAt: '2026-10-01T09:00:00Z',
			archivedAt: null,
			billingContactId: 'k1',
			billingAddressId: 'a2',
		})
	})

	it('defaults the optional details of a bare customer', () => {
		const details = accountDetailsOf(customerFixture({}), null)
		expect(details.kind).toBe('professionnel')
		expect(details.vatNumber).toBe('')
		expect(details.paymentTermsDays).toBeNull()
		expect(details.convertedAt).toBeNull()
		expect(details.stripeCustomerId).toBeNull()
	})
})

describe('directoryTerms', () => {
	it('gathers the searchable text and phones of a customer', () => {
		const terms = directoryTerms(
			[
				{
					id: 'k1',
					first_name: 'Léa',
					last_name: 'Duval',
					role: 'Gérante',
					email: 'lea@x.fr',
					phone: '02 40 12 34 56',
					mobile: '06 12 34 56 78',
				},
			],
			[{ city: 'Rezé' }],
		)
		expect(terms.text).toEqual(
			expect.arrayContaining(['Rezé', 'Léa Duval', 'lea@x.fr', 'Gérante']),
		)
		expect(terms.phones).toEqual(['02 40 12 34 56', '06 12 34 56 78'])
	})
})

describe('openDealsSentence', () => {
	it('words the fate of the open opportunities', () => {
		expect(openDealsSentence(null)).toBe(
			'Ses opportunités ouvertes seront marquées comme perdues (motif : compte archivé) et ne pourront pas être rouvertes.',
		)
		expect(openDealsSentence(0)).toBe(
			'Aucune opportunité ouverte n’est concernée.',
		)
		expect(openDealsSentence(1)).toBe(
			'Son opportunité ouverte sera marquée comme perdue (motif : compte archivé) et ne pourra pas être rouverte.',
		)
		expect(openDealsSentence(3)).toBe(
			'Ses 3 opportunités ouvertes seront marquées comme perdues (motif : compte archivé) et ne pourront pas être rouvertes.',
		)
	})
})
