import type { Customer } from '#/pages/customers/types'

export const MOCK_CUSTOMERS: Customer[] = [
	{
		id: 'c1f3a1d4-8b2e-4e3a-9f1a-1a2b3c4d5e6f',
		name: 'Cloud IAM',
		category: 'sme',
		contact_name: 'Nathael Bonnal',
		email: 'nathael.bonnal@cloud-iam.com',
		phone: '+33 6 00 00 00 01',
		address: { street: '12 rue de Paris', city: 'Toulouse', zip: '31000' },
		created_at: '2026-01-15T09:30:00.000Z',
	},
	{
		id: 'd2a4b3c5-9d3f-4a2b-8c1d-2b3c4d5e6f70',
		name: 'Plomberie Dupont',
		category: 'artisan',
		contact_name: 'Jean Dupont',
		email: 'contact@plomberie-dupont.fr',
		phone: '+33 6 12 34 56 78',
		address: { street: '5 avenue des Lilas', city: 'Lyon', zip: '69003' },
		created_at: '2026-02-02T14:10:00.000Z',
	},
	{
		id: 'e3b5c4d6-ae40-4b3c-9d2e-3c4d5e6f7081',
		name: 'Marie Leroy',
		category: 'individual',
		contact_name: 'Marie Leroy',
		email: 'marie.leroy@example.com',
		phone: '+33 7 98 76 54 32',
		address: { street: '8 impasse du Chêne', city: 'Bordeaux', zip: '33000' },
		created_at: '2026-03-11T08:45:00.000Z',
	},
]

export function findCustomerById(id: string): Customer | undefined {
	return MOCK_CUSTOMERS.find((c) => c.id === id)
}
