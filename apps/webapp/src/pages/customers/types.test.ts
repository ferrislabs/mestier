import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import {
	accountKindOf,
	addressKindOf,
	apiAddressKindOf,
	customerBody,
	customerKindOf,
	kindOf,
} from '#/pages/customers/types'

const customer: Customer = {
	id: 'c1',
	organization_id: 'org',
	name: 'Boulangerie Lemoine',
	status: 'PROSPECT',
	kind: 'PROFESSIONAL',
	registration_number: '12345678900012',
	email: 'a@b.fr',
	phone: '0240000000',
	website: 'https://x.fr',
	vat_number: 'FR123',
	owner_member_id: 'm1',
	lead_source: 'salon',
	notes: 'Note',
	payment_terms_days: 45,
	billing_contact_id: 'k1',
	billing_address_id: 'a1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

describe('kinds', () => {
	it('maps the customer kind both ways', () => {
		expect(accountKindOf('INDIVIDUAL')).toBe('particulier')
		expect(accountKindOf('PROFESSIONAL')).toBe('professionnel')
		expect(customerKindOf('particulier')).toBe('INDIVIDUAL')
		expect(customerKindOf('professionnel')).toBe('PROFESSIONAL')
		expect(kindOf(customer)).toBe('professionnel')
	})

	it('maps the address kind both ways', () => {
		for (const kind of ['BILLING', 'SITE', 'OTHER'] as const)
			expect(apiAddressKindOf(addressKindOf(kind))).toBe(kind)
		expect(addressKindOf('BILLING')).toBe('billing')
	})
})

describe('customerBody', () => {
	it('carries every field of the customer, the API replaces them all', () => {
		expect(customerBody(customer)).toEqual({
			status: 'PROSPECT',
			name: 'Boulangerie Lemoine',
			registration_number: '12345678900012',
			email: 'a@b.fr',
			phone: '0240000000',
			kind: 'PROFESSIONAL',
			website: 'https://x.fr',
			vat_number: 'FR123',
			owner_member_id: 'm1',
			lead_source: 'salon',
			notes: 'Note',
			payment_terms_days: 45,
			billing_contact_id: 'k1',
			billing_address_id: 'a1',
		})
	})

	it('keeps the profile when only the status changes', () => {
		const body = customerBody(customer, { status: 'ARCHIVED' })
		expect(body.status).toBe('ARCHIVED')
		expect(body.owner_member_id).toBe('m1')
		expect(body.billing_contact_id).toBe('k1')
	})

	it('sends null for what the customer does not have', () => {
		const body = customerBody({
			...customer,
			website: undefined,
			owner_member_id: null,
			payment_terms_days: undefined,
		})
		expect(body.website).toBeNull()
		expect(body.owner_member_id).toBeNull()
		expect(body.payment_terms_days).toBeNull()
	})

	it('lets an override clear a designation', () => {
		expect(
			customerBody(customer, { billing_contact_id: null }).billing_contact_id,
		).toBeNull()
	})
})
