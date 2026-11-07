import { describe, expect, it } from 'vitest'
import {
	eurosToCents,
	formatVatRateBp,
	type InvoiceLineFormValues,
	invoiceDisplayState,
	invoiceKindLabel,
	invoiceLineTotalCents,
	invoiceStatusLabel,
} from '#/pages/invoices/types'

function line(quantity: string, unitPrice: string): InvoiceLineFormValues {
	return {
		clientId: 'line-1',
		label: 'Taille de haie',
		quantity,
		unitPrice,
		vatRateBp: '',
		discountPercent: '0',
		productId: null,
		serviceRateId: null,
	}
}

describe('eurosToCents', () => {
	it('reads what an artisan types, comma or point', () => {
		expect(eurosToCents('45,00')).toBe(4500)
		expect(eurosToCents('45.00')).toBe(4500)
		expect(eurosToCents('118,45')).toBe(11845)
		expect(eurosToCents('0,05')).toBe(5)
	})

	it('treats unusable input as nothing rather than guessing', () => {
		expect(eurosToCents('')).toBe(0)
		expect(eurosToCents('abc')).toBe(0)
		expect(eurosToCents('.')).toBe(0)
	})
})

describe('invoiceLineTotalCents', () => {
	it('multiplies a quantity by a unit price', () => {
		expect(invoiceLineTotalCents(line('4', '45,00'))).toBe(18000)
		expect(invoiceLineTotalCents(line('12,5', '38,00'))).toBe(47500)
	})

	it('agrees with the backend on products that land on a half cent', () => {
		expect(invoiceLineTotalCents(line('631,9', '388,75'))).toBe(24_565_113)
		expect(invoiceLineTotalCents(line('1723,1', '54,55'))).toBe(9_399_511)
	})

	it('rounds a half away from zero, in both directions', () => {
		expect(invoiceLineTotalCents(line('0,5', '0,01'))).toBe(1)
		expect(invoiceLineTotalCents(line('1,5', '0,01'))).toBe(2)
		expect(invoiceLineTotalCents(line('2,5', '0,01'))).toBe(3)
		expect(invoiceLineTotalCents(line('2,5', '-0,01'))).toBe(-3)
	})

	it('applies the line discount before rounding', () => {
		expect(
			invoiceLineTotalCents({ ...line('1', '10,05'), discountPercent: '10' }),
		).toBe(905)
	})

	it('is nothing at all when the line has no quantity yet', () => {
		expect(invoiceLineTotalCents(line('', '45,00'))).toBe(0)
		expect(invoiceLineTotalCents(line('0', '45,00'))).toBe(0)
	})
})

describe('formatVatRateBp', () => {
	it('reads basis points as a percentage', () => {
		expect(formatVatRateBp(2000)).toBe('20 %')
		expect(formatVatRateBp(550)).toBe('5.50 %')
	})
})

describe('invoiceStatusLabel', () => {
	it('reads every status in French', () => {
		expect(invoiceStatusLabel('DRAFT')).toBe('Brouillon')
		expect(invoiceStatusLabel('ISSUED')).toBe('Émise')
		expect(invoiceStatusLabel('PAID')).toBe('Payée')
		expect(invoiceStatusLabel('PARTIALLY_PAID')).toBe('Partiellement payée')
		expect(invoiceStatusLabel('CANCELLED')).toBe('Annulée')
	})
})

describe('invoiceKindLabel', () => {
	it('reads every kind in French', () => {
		expect(invoiceKindLabel('STANDARD')).toBe('Facture')
		expect(invoiceKindLabel('DEPOSIT')).toBe('Acompte')
		expect(invoiceKindLabel('FINAL')).toBe('Solde')
		expect(invoiceKindLabel('CREDIT_NOTE')).toBe('Avoir')
	})
})

describe('invoiceDisplayState', () => {
	it('maps every state the backend decides to its label key', () => {
		expect(invoiceDisplayState({ display_state: 'DRAFT' })).toBe('draft')
		expect(invoiceDisplayState({ display_state: 'UNPAID' })).toBe('unpaid')
		expect(invoiceDisplayState({ display_state: 'PARTIAL' })).toBe('partial')
		expect(invoiceDisplayState({ display_state: 'OVERDUE' })).toBe('overdue')
		expect(invoiceDisplayState({ display_state: 'PAID' })).toBe('paid')
		expect(invoiceDisplayState({ display_state: 'CANCELLED' })).toBe(
			'cancelled',
		)
		expect(invoiceDisplayState({ display_state: 'REFUND_DUE' })).toBe(
			'refund_due',
		)
	})
})
