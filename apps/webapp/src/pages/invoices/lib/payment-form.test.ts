import { describe, expect, it } from 'vitest'
import {
	initialPaymentValues,
	paymentErrors,
	paymentRequest,
	paymentsAndRefunds,
	todayDate,
} from './payment-form'

const TODAY = '2026-06-10'

describe('todayDate', () => {
	it('reads the local calendar day', () => {
		expect(todayDate(new Date(2026, 0, 5, 23, 30))).toBe('2026-01-05')
	})
})

describe('initialPaymentValues', () => {
	it('proposes the amount that remains to be collected', () => {
		expect(initialPaymentValues(12050, TODAY)).toMatchObject({
			date: TODAY,
			amount: '120,50',
			method: 'TRANSFER',
		})
	})

	it('proposes nothing when the balance is unknown or settled', () => {
		expect(initialPaymentValues(null, TODAY).amount).toBe('')
		expect(initialPaymentValues(0, TODAY).amount).toBe('')
	})
})

describe('paymentErrors', () => {
	const values = {
		date: TODAY,
		amount: '50',
		method: 'TRANSFER' as const,
		reference: '',
		note: '',
	}

	it('accepts a payment that fits the balance', () => {
		expect(paymentErrors(values, 12000, TODAY)).toEqual([])
	})

	it('refuses a future date, a null amount and an amount above the balance', () => {
		expect(
			paymentErrors({ ...values, date: '2026-06-11' }, 12000, TODAY),
		).toEqual(['La date du paiement ne peut pas être dans le futur.'])
		expect(paymentErrors({ ...values, amount: '' }, 12000, TODAY)).toEqual([
			'Saisis un montant supérieur à zéro.',
		])
		expect(paymentErrors({ ...values, amount: '500' }, 12000, TODAY)).toEqual([
			expect.stringContaining('dépasse le reste dû'),
		])
	})

	it('does not check the balance while it is unknown', () => {
		expect(paymentErrors({ ...values, amount: '500' }, null, TODAY)).toEqual([])
	})
})

describe('paymentRequest', () => {
	it('sends the amount in cents and drops empty optional fields', () => {
		expect(
			paymentRequest({
				date: TODAY,
				amount: '120,50',
				method: 'CHECK' as const,
				reference: '',
				note: ' ok ',
			}),
		).toEqual({
			amount_cents: 12050,
			paid_on: TODAY,
			method: 'CHECK',
			reference: null,
			note: 'ok',
		})
	})
})

describe('refund mode', () => {
	const values = {
		date: TODAY,
		amount: '30',
		method: 'TRANSFER' as const,
		reference: '',
		note: '',
	}

	it('proposes the over-payment that is owed back', () => {
		expect(initialPaymentValues(-4500, TODAY, 'refund').amount).toBe('45,00')
		expect(initialPaymentValues(12000, TODAY, 'refund').amount).toBe('')
	})

	it('caps a refund at the over-payment', () => {
		expect(paymentErrors(values, -4500, TODAY, 'refund')).toEqual([])
		expect(
			paymentErrors({ ...values, amount: '50' }, -4500, TODAY, 'refund'),
		).toEqual([expect.stringContaining('dépasse le trop-perçu')])
		expect(paymentErrors(values, 12000, TODAY, 'refund')).toEqual([
			expect.stringContaining('dépasse le trop-perçu'),
		])
	})

	it('sends a refund as a negative amount', () => {
		expect(paymentRequest({ ...values, amount: '30,50' }, 'refund')).toEqual({
			amount_cents: -3050,
			paid_on: TODAY,
			method: 'TRANSFER',
			reference: null,
			note: null,
		})
	})
})

describe('paymentsAndRefunds', () => {
	const payment = (id: string, amount: number) =>
		({
			id,
			amount_cents: amount,
			paid_on: TODAY,
			method: 'CASH',
			reference: null,
			note: null,
		}) as never

	it('splits the recorded payments from the refunds, a refund being a negative payment', () => {
		const split = paymentsAndRefunds([payment('a', 5000), payment('b', -1200)])

		expect(split.payments.map((item) => item.id)).toEqual(['a'])
		expect(split.refunds).toEqual([
			{
				id: 'b',
				date: TODAY,
				amountCents: 1200,
				method: 'CASH',
				reference: null,
				note: null,
			},
		])
	})
})
