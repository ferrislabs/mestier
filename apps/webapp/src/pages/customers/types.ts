export type CustomerCategory = 'artisan' | 'sme' | 'individual'

export interface CustomerAddress {
	street: string
	city: string
	zip: string
}

export interface Customer {
	id: string
	name: string
	category: CustomerCategory
	contact_name: string
	email: string
	phone: string
	address: CustomerAddress
	created_at: string
}

export const CATEGORY_LABELS: Record<CustomerCategory, string> = {
	artisan: 'Artisan',
	sme: 'PME',
	individual: 'Particulier',
}
