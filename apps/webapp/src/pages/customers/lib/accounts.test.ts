import { describe, expect, it } from 'vitest'
import { EMPTY_EXTRAS } from '#/pages/customers/feature/mocks/account-extras'
import {
	billingContactOf,
	billingContextOf,
	contactName,
	contextCountry,
	contextKind,
	directoryTerms,
	mainContactOf,
	openDealsSentence,
	withEntry,
} from '#/pages/customers/lib/accounts'

const contacts = [
	{ id: 'k1', first_name: 'Léa', last_name: 'Duval', is_primary: false },
	{ id: 'k2', first_name: 'Paul', last_name: 'Roux', is_primary: true },
]

const contexts = [
	{ id: 'a1', label: 'Chantier', city: 'Rezé' },
	{ id: 'a2', label: 'Siège', city: 'Nantes' },
]

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
		expect(
			billingContactOf(contacts, { ...EMPTY_EXTRAS, billingContactId: 'k1' })
				?.id,
		).toBe('k1')
		expect(
			billingContactOf(contacts, { ...EMPTY_EXTRAS, billingContactId: 'gone' })
				?.id,
		).toBe('k2')
		expect(billingContactOf(contacts, EMPTY_EXTRAS)?.id).toBe('k2')
	})
})

describe('addresses', () => {
	it('prefers the designated billing address, then a billing-kind one, then the first', () => {
		expect(
			billingContextOf(contexts, { ...EMPTY_EXTRAS, billingContextId: 'a2' })
				?.id,
		).toBe('a2')
		expect(
			billingContextOf(contexts, {
				...EMPTY_EXTRAS,
				addressKinds: { a2: 'billing' },
			})?.id,
		).toBe('a2')
		expect(billingContextOf(contexts, EMPTY_EXTRAS)?.id).toBe('a1')
		expect(billingContextOf([], EMPTY_EXTRAS)).toBeUndefined()
	})

	it('defaults the kind and the country', () => {
		expect(contextKind('a1', EMPTY_EXTRAS)).toBe('site')
		expect(
			contextKind('a1', { ...EMPTY_EXTRAS, addressKinds: { a1: 'other' } }),
		).toBe('other')
		expect(contextCountry('a1', EMPTY_EXTRAS)).toBe('France')
		expect(
			contextCountry('a1', {
				...EMPTY_EXTRAS,
				addressCountries: { a1: 'Belgique' },
			}),
		).toBe('Belgique')
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
				},
			],
			[{ city: 'Rezé' }],
			{ ...EMPTY_EXTRAS, mobiles: { k1: '06 12 34 56 78' } },
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

describe('withEntry', () => {
	it('sets, replaces and drops an entry without touching the source', () => {
		const source = { a: '1' }
		expect(withEntry(source, 'b', '2')).toEqual({ a: '1', b: '2' })
		expect(withEntry(source, 'a', '9')).toEqual({ a: '9' })
		expect(withEntry(source, 'a', '')).toEqual({})
		expect(source).toEqual({ a: '1' })
	})
})
