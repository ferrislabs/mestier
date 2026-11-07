import { describe, expect, it } from 'vitest'
import type { Supplier } from '#/hooks/use-suppliers'
import {
	addressFromText,
	addressText,
	buildSupplierRequest,
	emptyAddress,
	emptyContact,
	emptySupplierValues,
	extrasOfSupplier,
	extrasOfValues,
	firstAddressLine,
	supplierFormErrors,
	supplierValuesOf,
	validateSupplierName,
} from '#/pages/suppliers/lib/supplier-form'
import type { SupplierFormValues } from '#/pages/suppliers/types'
import { EMPTY_SUPPLIER_EXTRAS } from '#/pages/suppliers/types'

function supplier(overrides: Partial<Supplier>): Supplier {
	return {
		id: 's1',
		organization_id: 'org',
		name: 'Pointp',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	} as Supplier
}

function filled(patch: Partial<SupplierFormValues>): SupplierFormValues {
	return { ...emptySupplierValues(), name: 'Pointp', ...patch }
}

describe('emptySupplierValues', () => {
	it('starts with one blank contact and one blank address', () => {
		const values = emptySupplierValues()
		expect(values.contacts).toHaveLength(1)
		expect(values.addresses).toHaveLength(1)
		expect(values.addresses[0]?.kind).toBe('billing')
		expect(values.defaultCategoryId).toBeNull()
	})
})

describe('supplierValuesOf', () => {
	it('reads the API fields and falls back on a contact and an address', () => {
		const values = supplierValuesOf(
			supplier({
				registration_number: '82345678900019',
				email: 'a@b.fr',
				phone: '0102030405',
				address: '12 rue des Lilas\nZA Nord\n44000 Nantes',
			}),
			EMPTY_SUPPLIER_EXTRAS,
		)
		expect(values.registrationNumber).toBe('82345678900019')
		expect(values.contacts[0]).toMatchObject({
			email: 'a@b.fr',
			phone: '0102030405',
		})
		expect(values.addresses[0]).toMatchObject({
			street: '12 rue des Lilas',
			complement: 'ZA Nord',
			postalCode: '44000',
			city: 'Nantes',
		})
	})

	it('prefers the extras when they carry contacts and addresses', () => {
		const contact = { ...emptyContact('c1'), firstName: 'Gaëtan' }
		const address = { ...emptyAddress('a1'), street: '1 rue X' }
		const values = supplierValuesOf(supplier({ email: 'ignored@x.fr' }), {
			...EMPTY_SUPPLIER_EXTRAS,
			iban: 'FR76 1',
			paymentTermsDays: 30,
			defaultCategoryId: 'cat-materiaux',
			notes: 'Livraison le mardi',
			contacts: [contact],
			addresses: [address],
		})
		expect(values.contacts).toEqual([contact])
		expect(values.addresses).toEqual([address])
		expect(values.iban).toBe('FR76 1')
		expect(values.paymentTermsDays).toBe('30')
		expect(values.defaultCategoryId).toBe('cat-materiaux')
		expect(values.notes).toBe('Livraison le mardi')
	})
})

describe('supplierFormErrors', () => {
	it('accepts a filled form', () => {
		expect(supplierFormErrors(filled({}))).toEqual([])
	})

	it('requires a name', () => {
		expect(supplierFormErrors(filled({ name: '  ' }))).toContain(
			'Le nom est obligatoire.',
		)
		expect(validateSupplierName('A')).toBeUndefined()
	})

	it('checks the SIRET length, spaces ignored, and still accepts a SIREN', () => {
		expect(
			supplierFormErrors(filled({ registrationNumber: '823 456 789 00019' })),
		).toEqual([])
		expect(
			supplierFormErrors(filled({ registrationNumber: '823456789' })),
		).toEqual([])
		expect(supplierFormErrors(filled({ registrationNumber: '12345' }))).toEqual(
			['Le SIRET (14 chiffres) n’est pas valide.'],
		)
	})

	it('checks every contact e-mail that is filled', () => {
		const bad = { ...emptyContact('c2'), email: 'pas-un-mail' }
		expect(
			supplierFormErrors(filled({ contacts: [emptyContact('c1'), bad] })),
		).toEqual(['L’adresse e-mail du contact 2 n’est pas valide.'])
	})

	it('wants a whole number of days for the payment terms', () => {
		expect(supplierFormErrors(filled({ paymentTermsDays: '30' }))).toEqual([])
		expect(supplierFormErrors(filled({ paymentTermsDays: '' }))).toEqual([])
		expect(supplierFormErrors(filled({ paymentTermsDays: '3,5' }))).toEqual([
			'Le délai de paiement doit être un nombre de jours entier.',
		])
	})
})

describe('buildSupplierRequest', () => {
	it('sends blanks as null', () => {
		expect(buildSupplierRequest(filled({ name: ' Martin ' }))).toEqual({
			name: 'Martin',
			registration_number: null,
			vat_number: null,
			address: null,
			phone: null,
			email: null,
			iban: null,
			payment_terms_days: null,
			default_category_id: null,
			notes: null,
			contacts: [],
			addresses: [],
		})
	})

	it('takes the e-mail, phone and address from the first filled entries', () => {
		const request = buildSupplierRequest(
			filled({
				registrationNumber: '823 456 789 00019',
				contacts: [
					emptyContact('c1'),
					{
						...emptyContact('c2'),
						email: ' a@b.fr ',
						phone: '0102030405',
					},
				],
				addresses: [
					{
						...emptyAddress('a1'),
						street: '12 rue des Lilas',
						postalCode: '44000',
						city: 'Nantes',
					},
				],
			}),
		)
		expect(request).toMatchObject({
			registration_number: '82345678900019',
			email: 'a@b.fr',
			phone: '0102030405',
			address: '12 rue des Lilas\n44000 Nantes',
		})
	})
})

describe('buildSupplierRequest profile', () => {
	it('sends the IBAN, the terms, the category, the notes, the contacts and the typed addresses', () => {
		const request = buildSupplierRequest(
			filled({
				iban: ' FR76 3000 ',
				paymentTermsDays: '45',
				defaultCategoryId: 'c1',
				notes: ' Livraison le mardi ',
				contacts: [
					{
						...emptyContact('c1'),
						firstName: 'Jean',
						lastName: 'Bernard',
						role: 'Commercial',
						mobile: '0600000000',
					},
				],
				addresses: [
					{
						...emptyAddress('a1'),
						kind: 'site',
						street: '2 rue B',
						complement: 'Bât. 3',
						postalCode: '44800',
						city: 'Saint-Herblain',
					},
				],
			}),
		)

		expect(request).toMatchObject({
			iban: 'FR76 3000',
			payment_terms_days: 45,
			default_category_id: 'c1',
			notes: 'Livraison le mardi',
			contacts: [
				{
					first_name: 'Jean',
					last_name: 'Bernard',
					role: 'Commercial',
					email: '',
					phone: '',
					mobile: '0600000000',
				},
			],
			addresses: [
				{
					kind: 'SITE',
					line2: 'Bât. 3',
					postal_code: '44800',
					city: 'Saint-Herblain',
					country: 'France',
				},
			],
		})
	})
})

describe('extrasOfSupplier', () => {
	it('reads the profile, the typed addresses and the archive state from the API', () => {
		const extras = extrasOfSupplier(
			supplier({
				iban: 'FR76',
				payment_terms_days: 30,
				default_category_id: 'c1',
				notes: 'RAS',
				archived_at: '2026-11-02T00:00:00Z',
				contacts: [
					{
						id: 'k1',
						first_name: 'Jean',
						last_name: 'Bernard',
						role: '',
						email: 'j@b.fr',
						phone: '',
						mobile: '06',
					},
				],
				addresses: [
					{
						id: 'a1',
						kind: 'BILLING',
						label: 'Siège',
						street: '1 rue A',
						line2: 'Bât. 1',
						postal_code: '44000',
						city: 'Nantes',
						country: 'France',
					},
				],
			}),
		)

		expect(extras).toMatchObject({
			iban: 'FR76',
			paymentTermsDays: 30,
			defaultCategoryId: 'c1',
			notes: 'RAS',
			archived: true,
		})
		expect(extras.contacts?.[0]).toMatchObject({
			firstName: 'Jean',
			mobile: '06',
		})
		expect(extras.addresses?.[0]).toMatchObject({
			kind: 'billing',
			complement: 'Bât. 1',
			postalCode: '44000',
		})
	})

	it('falls back to null contacts and addresses when none are stored', () => {
		const extras = extrasOfSupplier(supplier({ contacts: [], addresses: [] }))

		expect(extras.contacts).toBeNull()
		expect(extras.addresses).toBeNull()
		expect(extras.archived).toBe(false)
		expect(extras.iban).toBe('')
	})
})

describe('extrasOfValues', () => {
	it('drops blank contacts and addresses and parses the payment terms', () => {
		const contact = { ...emptyContact('c2'), lastName: 'Rivière' }
		const extras = extrasOfValues(
			filled({
				iban: ' FR76 1 ',
				paymentTermsDays: '45',
				contacts: [emptyContact('c1'), contact],
			}),
		)
		expect(extras.iban).toBe('FR76 1')
		expect(extras.paymentTermsDays).toBe(45)
		expect(extras.contacts).toEqual([contact])
		expect(extras.addresses).toEqual([])
	})

	it('keeps an empty delay as null', () => {
		expect(extrasOfValues(filled({})).paymentTermsDays).toBeNull()
	})
})

describe('address text', () => {
	it('round-trips street, complement, postal code and city', () => {
		const address = {
			...emptyAddress('a'),
			street: '1 rue X',
			complement: 'Bât B',
			postalCode: '75000',
			city: 'Paris',
		}
		expect(addressText(address)).toBe('1 rue X\nBât B\n75000 Paris')
		expect(addressFromText(addressText(address), 'a')).toMatchObject({
			street: '1 rue X',
			complement: 'Bât B',
			postalCode: '75000',
			city: 'Paris',
		})
	})

	it('returns nothing for a blank text', () => {
		expect(addressFromText('  \n ', 'a')).toBeNull()
		expect(addressFromText(null, 'a')).toBeNull()
	})

	it('keeps the first line for the list', () => {
		expect(firstAddressLine('1 rue X\n75000 Paris')).toBe('1 rue X')
		expect(firstAddressLine(null)).toBe('')
	})
})
