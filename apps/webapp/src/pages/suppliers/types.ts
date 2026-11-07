export type AddressKind = 'billing' | 'site' | 'other'

export const ADDRESS_KIND_LABELS: Record<AddressKind, string> = {
	billing: 'Facturation',
	site: 'Site',
	other: 'Autre',
}

export const ADDRESS_KINDS: readonly AddressKind[] = [
	'billing',
	'site',
	'other',
]

export interface SupplierContact {
	id: string
	firstName: string
	lastName: string
	role: string
	email: string
	phone: string
	mobile: string
}

export interface SupplierAddress {
	id: string
	label: string
	kind: AddressKind
	street: string
	complement: string
	postalCode: string
	city: string
	country: string
}

export interface SupplierExtras {
	iban: string
	defaultCategoryId: string | null
	paymentTermsDays: number | null
	notes: string
	archived: boolean
	contacts: SupplierContact[] | null
	addresses: SupplierAddress[] | null
}

export interface SupplierFormValues {
	name: string
	registrationNumber: string
	vatNumber: string
	iban: string
	defaultCategoryId: string | null
	paymentTermsDays: string
	notes: string
	contacts: SupplierContact[]
	addresses: SupplierAddress[]
}

export const EMPTY_SUPPLIER_EXTRAS: SupplierExtras = {
	iban: '',
	defaultCategoryId: null,
	paymentTermsDays: null,
	notes: '',
	archived: false,
	contacts: null,
	addresses: null,
}
