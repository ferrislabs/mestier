import { describe, expect, it } from 'vitest'
import {
	eurosToCents,
	formatVatRateBp,
	type InvoiceLineFormValues,
	invoiceDisplayState,
	invoiceKindLabel,
	invoiceLineTotalCents,
	invoiceStatusLabel,
	isInvoiceOverdue,
} from '#/pages/invoices/types'

function line(quantity: string, unitPrice: string): InvoiceLineFormValues {
	return {
		clientId: 'line-1',
		label: 'Taille de haie',
		quantity,
		unitPrice,
		vatRateBp: '',
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

	/** Same vectors `pages/quotes/types.test.ts` pins for `quoteLineTotalCents`
	 * — the two implementations share the rounding rule, not the code, so both
	 * are pinned independently against the backend's own values. */
	it('agrees with the backend on products that land on a half cent', () => {
		expect(invoiceLineTotalCents(line('631,9', '388,75'))).toBe(24_565_112)
		expect(invoiceLineTotalCents(line('1723,1', '54,55'))).toBe(9_399_510)
	})

	it('rounds a half to its even neighbour, in both directions', () => {
		expect(invoiceLineTotalCents(line('0,5', '0,01'))).toBe(0)
		expect(invoiceLineTotalCents(line('1,5', '0,01'))).toBe(2)
		expect(invoiceLineTotalCents(line('2,5', '0,01'))).toBe(2)
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

describe('isInvoiceOverdue', () => {
	const now = new Date('2026-08-23T00:00:00Z')

	it('is overdue only when issued, due, and the due date has passed', () => {
		expect(
			isInvoiceOverdue(
				{ status: 'ISSUED', due_at: '2026-08-01T00:00:00Z' },
				now,
			),
		).toBe(true)
	})

	it('is not overdue before the due date', () => {
		expect(
			isInvoiceOverdue(
				{ status: 'ISSUED', due_at: '2026-09-01T00:00:00Z' },
				now,
			),
		).toBe(false)
	})

	it('is not overdue without a due date', () => {
		expect(isInvoiceOverdue({ status: 'ISSUED', due_at: null }, now)).toBe(
			false,
		)
	})

	it('is not overdue once paid, even past the due date', () => {
		expect(
			isInvoiceOverdue({ status: 'PAID', due_at: '2026-08-01T00:00:00Z' }, now),
		).toBe(false)
	})

	it('is not overdue on a draft', () => {
		expect(
			isInvoiceOverdue(
				{ status: 'DRAFT', due_at: '2026-08-01T00:00:00Z' },
				now,
			),
		).toBe(false)
	})
})

describe('invoiceDisplayState', () => {
	const now = new Date('2026-06-10T12:00:00Z')
	const future = '2026-07-01T00:00:00Z'
	const past = '2026-06-01T00:00:00Z'

	it('reads a draft and a cancelled invoice as they are', () => {
		expect(
			invoiceDisplayState({ status: 'DRAFT', due_at: past }, null, now),
		).toBe('draft')
		expect(
			invoiceDisplayState({ status: 'CANCELLED', due_at: past }, 0, now),
		).toBe('cancelled')
	})

	it('reads an issued invoice as unpaid, or overdue once its due date passed', () => {
		expect(
			invoiceDisplayState({ status: 'ISSUED', due_at: future }, null, now),
		).toBe('unpaid')
		expect(
			invoiceDisplayState({ status: 'ISSUED', due_at: past }, null, now),
		).toBe('overdue')
		expect(
			invoiceDisplayState({ status: 'ISSUED', due_at: null }, null, now),
		).toBe('unpaid')
	})

	it('keeps a partially paid invoice partial until it is late', () => {
		expect(
			invoiceDisplayState(
				{ status: 'PARTIALLY_PAID', due_at: future },
				5000,
				now,
			),
		).toBe('partial')
		expect(
			invoiceDisplayState(
				{ status: 'PARTIALLY_PAID', due_at: past },
				5000,
				now,
			),
		).toBe('overdue')
	})

	it('never calls a paid invoice late', () => {
		expect(invoiceDisplayState({ status: 'PAID', due_at: past }, 0, now)).toBe(
			'paid',
		)
	})

	it('flags an overpayment as a refund due', () => {
		expect(
			invoiceDisplayState({ status: 'PAID', due_at: past }, -2000, now),
		).toBe('refund_due')
	})
})
