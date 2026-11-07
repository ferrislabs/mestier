import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import {
	accountFormOf,
	emptyAccountForm,
	parsePaymentTerms,
	profileOfForm,
	registrationNumberOf,
	stripeCustomerIdOf,
	validateAccountForm,
} from '#/pages/customers/lib/account-form'

describe('validateAccountForm', () => {
	it('requires a name', () => {
		expect(validateAccountForm(emptyAccountForm(null), 'create').name).toBe(
			'Le nom est obligatoire.',
		)
	})

	it('accepts a bare account in create mode without contact or address', () => {
		const form = { ...emptyAccountForm(null), name: 'Duval' }
		expect(validateAccountForm(form, 'create')).toEqual({})
	})

	it('requires both names once a contact field is filled', () => {
		const form = {
			...emptyAccountForm(null),
			name: 'Duval',
			contact: {
				firstName: 'Léa',
				lastName: '',
				role: '',
				email: 'lea',
				mobile: '12',
				phone: '',
			},
		}
		const errors = validateAccountForm(form, 'create')
		expect(errors.contactName).toBeDefined()
		expect(errors.contactEmail).toBeDefined()
		expect(errors.contactMobile).toBeDefined()
	})

	it('validates an address only when one is started', () => {
		const form = {
			...emptyAccountForm(null),
			name: 'Duval',
			address: {
				street: '1 rue X',
				complement: '',
				postalCode: '4480',
				city: '',
				country: 'France',
			},
		}
		const errors = validateAccountForm(form, 'create')
		expect(errors.postalCode).toBeDefined()
		expect(errors.city).toBeDefined()
		expect(errors.street).toBeUndefined()
	})

	it('checks the SIRET of a professional, legacy SIREN included', () => {
		const form = { ...emptyAccountForm(null), name: 'Duval' }
		expect(
			validateAccountForm({ ...form, registrationNumber: '8234567' }, 'edit')
				.registrationNumber,
		).toBe('Le SIRET doit comporter 14 chiffres.')
		expect(
			validateAccountForm(
				{ ...form, registrationNumber: '823 456 789 00012' },
				'edit',
			),
		).toEqual({})
		expect(
			validateAccountForm({ ...form, registrationNumber: '823456789' }, 'edit'),
		).toEqual({})
		expect(
			validateAccountForm(
				{ ...form, kind: 'particulier', registrationNumber: 'abc' },
				'edit',
			),
		).toEqual({})
	})

	it('keeps the default pipeline stage on a new account', () => {
		expect(emptyAccountForm(null).pipelineStage).toBe('NEW')
	})

	it('checks payment terms and the Stripe customer id', () => {
		const form = {
			...emptyAccountForm(null),
			name: 'Duval',
			paymentTerms: '90',
			stripeCustomerId: 'abc',
		}
		const errors = validateAccountForm(form, 'edit')
		expect(errors.paymentTerms).toBeDefined()
		expect(errors.stripeCustomerId).toBeDefined()
		expect(
			validateAccountForm(
				{ ...form, paymentTerms: '30', stripeCustomerId: 'cus_ABC123' },
				'edit',
			),
		).toEqual({})
	})
})

describe('parsePaymentTerms', () => {
	it('maps blank to null and garbage to NaN', () => {
		expect(parsePaymentTerms('  ')).toBeNull()
		expect(parsePaymentTerms('45')).toBe(45)
		expect(parsePaymentTerms('4x')).toBeNaN()
	})
})

describe('registrationNumberOf', () => {
	it('strips spaces for a professional and drops it for a private person', () => {
		const form = {
			...emptyAccountForm(null),
			registrationNumber: '823 456 789 00019',
		}
		expect(registrationNumberOf(form)).toBe('82345678900019')
		expect(registrationNumberOf({ ...form, kind: 'particulier' })).toBeNull()
		expect(
			registrationNumberOf({ ...form, registrationNumber: ' ' }),
		).toBeNull()
	})
})

describe('profileOfForm', () => {
	it('sends the customer profile in the API vocabulary', () => {
		const form = {
			...emptyAccountForm('m1'),
			kind: 'particulier' as const,
			vatNumber: ' FR123 ',
			website: ' https://x.fr ',
			source: 'salon' as const,
			paymentTerms: '45',
			notes: ' Livraison le matin ',
		}
		expect(profileOfForm(form)).toEqual({
			kind: 'INDIVIDUAL',
			website: 'https://x.fr',
			vat_number: 'FR123',
			owner_member_id: 'm1',
			lead_source: 'salon',
			notes: 'Livraison le matin',
			payment_terms_days: 45,
		})
	})

	it('sends blank texts and terms as null', () => {
		expect(profileOfForm(emptyAccountForm(null))).toEqual({
			kind: 'PROFESSIONAL',
			website: null,
			vat_number: null,
			owner_member_id: null,
			lead_source: null,
			notes: null,
			payment_terms_days: null,
		})
	})
})

describe('accountFormOf', () => {
	const customer: Customer = {
		id: 'c1',
		organization_id: 'org',
		name: 'Boulangerie Lemoine',
		status: 'CLIENT',
		pipeline_stage: 'WON',
		kind: 'INDIVIDUAL',
		registration_number: null,
		vat_number: 'FR123',
		website: 'https://x.fr',
		owner_member_id: 'm1',
		lead_source: 'recommandation',
		payment_terms_days: 0,
		notes: 'Note',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}

	it('fills the form from the customer and the Stripe identifier', () => {
		const form = accountFormOf(customer, 'cus_1')
		expect(form.kind).toBe('particulier')
		expect(form.status).toBe('client')
		expect(form.vatNumber).toBe('FR123')
		expect(form.website).toBe('https://x.fr')
		expect(form.ownerId).toBe('m1')
		expect(form.source).toBe('recommandation')
		expect(form.paymentTerms).toBe('0')
		expect(form.notes).toBe('Note')
		expect(form.stripeCustomerId).toBe('cus_1')
	})

	it('leaves the terms blank when the customer has none', () => {
		const form = accountFormOf(
			{ ...customer, payment_terms_days: null, owner_member_id: null },
			null,
		)
		expect(form.paymentTerms).toBe('')
		expect(form.ownerId).toBeNull()
		expect(form.stripeCustomerId).toBe('')
	})
})

describe('stripeCustomerIdOf', () => {
	it('trims the identifier and maps blank to null', () => {
		const form = emptyAccountForm(null)
		expect(stripeCustomerIdOf({ ...form, stripeCustomerId: ' cus_1 ' })).toBe(
			'cus_1',
		)
		expect(stripeCustomerIdOf(form)).toBeNull()
	})
})
