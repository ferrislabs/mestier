import type { Supplier } from '#/hooks/use-suppliers'
import type {
	SupplierAddress,
	SupplierContact,
	SupplierExtras,
	SupplierFormValues,
} from '#/pages/suppliers/types'

const TERMS = /^\d{1,3}$/
const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
const POSTAL_LINE = /^(\d{5})\s+(.+)$/

export function emptyContact(id: string): SupplierContact {
	return {
		id,
		firstName: '',
		lastName: '',
		role: '',
		email: '',
		phone: '',
		mobile: '',
	}
}

export function emptyAddress(id: string): SupplierAddress {
	return {
		id,
		label: 'Siège',
		kind: 'billing',
		street: '',
		complement: '',
		postalCode: '',
		city: '',
		country: 'France',
	}
}

export function emptySupplierValues(): SupplierFormValues {
	return {
		name: '',
		registrationNumber: '',
		vatNumber: '',
		iban: '',
		defaultCategoryId: null,
		paymentTermsDays: '',
		notes: '',
		contacts: [emptyContact('contact-new')],
		addresses: [emptyAddress('address-new')],
	}
}

export function isContactEmpty(contact: SupplierContact): boolean {
	return (
		contact.firstName.trim() === '' &&
		contact.lastName.trim() === '' &&
		contact.email.trim() === '' &&
		contact.phone.trim() === '' &&
		contact.mobile.trim() === ''
	)
}

export function isAddressEmpty(address: SupplierAddress): boolean {
	return (
		address.street.trim() === '' &&
		address.postalCode.trim() === '' &&
		address.city.trim() === ''
	)
}

export function contactName(
	contact: Pick<SupplierContact, 'firstName' | 'lastName'>,
): string {
	return `${contact.firstName} ${contact.lastName}`.trim()
}

export function addressText(address: SupplierAddress): string {
	const town = [address.postalCode.trim(), address.city.trim()]
		.filter(Boolean)
		.join(' ')
	return [address.street.trim(), address.complement.trim(), town]
		.filter((line) => line !== '')
		.join('\n')
}

export function addressFromText(
	text: string | null | undefined,
	id: string,
): SupplierAddress | null {
	const lines = (text ?? '')
		.split('\n')
		.map((line) => line.trim())
		.filter((line) => line !== '')
	if (lines.length === 0) return null
	const postal = POSTAL_LINE.exec(lines.at(-1) ?? '')
	const rest = postal ? lines.slice(0, -1) : lines
	return {
		...emptyAddress(id),
		street: rest[0] ?? '',
		complement: rest.slice(1).join(', '),
		postalCode: postal?.[1] ?? '',
		city: postal?.[2] ?? '',
	}
}

export function firstAddressLine(address: string | null | undefined): string {
	return address?.split('\n')[0]?.trim() ?? ''
}

export function validateSupplierName(name: string): string | undefined {
	return name.trim().length === 0 ? 'Le nom est obligatoire.' : undefined
}

function parseTerms(input: string): number | null | undefined {
	const trimmed = input.trim()
	if (trimmed === '') return null
	return TERMS.test(trimmed) ? Number(trimmed) : undefined
}

export function supplierValuesOf(
	supplier: Supplier,
	extras: SupplierExtras,
): SupplierFormValues {
	const fromApi: SupplierContact[] =
		supplier.email || supplier.phone
			? [
					{
						...emptyContact('contact-api'),
						email: supplier.email ?? '',
						phone: supplier.phone ?? '',
					},
				]
			: []
	const contacts = extras.contacts ?? fromApi
	const addressFromApi = addressFromText(supplier.address, 'address-api')
	const addresses = extras.addresses ?? (addressFromApi ? [addressFromApi] : [])
	return {
		name: supplier.name,
		registrationNumber: supplier.registration_number ?? '',
		vatNumber: supplier.vat_number ?? '',
		iban: extras.iban,
		defaultCategoryId: extras.defaultCategoryId,
		paymentTermsDays:
			extras.paymentTermsDays === null ? '' : String(extras.paymentTermsDays),
		notes: extras.notes,
		contacts: contacts.length > 0 ? contacts : [emptyContact('contact-new')],
		addresses: addresses.length > 0 ? addresses : [emptyAddress('address-new')],
	}
}

export function supplierFormErrors(values: SupplierFormValues): string[] {
	const errors: string[] = []
	const nameError = validateSupplierName(values.name)
	if (nameError) errors.push(nameError)
	const legalId = values.registrationNumber.replace(/\s/g, '')
	if (legalId !== '' && !/^\d{9}(\d{5})?$/.test(legalId))
		errors.push('Le SIRET (14 chiffres) n’est pas valide.')
	values.contacts.forEach((contact, index) => {
		const email = contact.email.trim()
		if (email !== '' && !EMAIL.test(email))
			errors.push(`L’adresse e-mail du contact ${index + 1} n’est pas valide.`)
	})
	if (parseTerms(values.paymentTermsDays) === undefined)
		errors.push('Le délai de paiement doit être un nombre de jours entier.')
	return errors
}

function optional(value: string): string | null {
	const trimmed = value.trim()
	return trimmed.length > 0 ? trimmed : null
}

export function buildSupplierRequest(values: SupplierFormValues) {
	const address = values.addresses.find((item) => !isAddressEmpty(item))
	return {
		name: values.name.trim(),
		registration_number: optional(values.registrationNumber.replace(/\s/g, '')),
		vat_number: optional(values.vatNumber),
		address: address ? addressText(address) : null,
		phone: optional(
			values.contacts.find((item) => item.phone.trim() !== '')?.phone ?? '',
		),
		email: optional(
			values.contacts.find((item) => item.email.trim() !== '')?.email ?? '',
		),
	}
}

function cleanContact(contact: SupplierContact): SupplierContact {
	return {
		id: contact.id,
		firstName: contact.firstName.trim(),
		lastName: contact.lastName.trim(),
		role: contact.role.trim(),
		email: contact.email.trim(),
		phone: contact.phone.trim(),
		mobile: contact.mobile.trim(),
	}
}

function cleanAddress(address: SupplierAddress): SupplierAddress {
	return {
		id: address.id,
		label: address.label.trim() || 'Adresse',
		kind: address.kind,
		street: address.street.trim(),
		complement: address.complement.trim(),
		postalCode: address.postalCode.trim(),
		city: address.city.trim(),
		country: address.country.trim() || 'France',
	}
}

export function extrasOfValues(
	values: SupplierFormValues,
): Omit<SupplierExtras, 'archived'> {
	return {
		iban: values.iban.trim(),
		defaultCategoryId: values.defaultCategoryId,
		paymentTermsDays: parseTerms(values.paymentTermsDays) ?? null,
		notes: values.notes.trim(),
		contacts: values.contacts
			.filter((contact) => !isContactEmpty(contact))
			.map(cleanContact),
		addresses: values.addresses
			.filter((address) => !isAddressEmpty(address))
			.map(cleanAddress),
	}
}
