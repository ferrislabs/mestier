import { describe, expect, it } from 'vitest'
import {
	emptyAccountForm,
	parsePaymentTerms,
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
				phone: '',
			},
		}
		const errors = validateAccountForm(form, 'create')
		expect(errors.contactName).toBeDefined()
		expect(errors.contactEmail).toBeDefined()
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
