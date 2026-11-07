import { describe, expect, it } from 'vitest'
import type { Invoice, InvoicePayment } from '#/hooks/use-invoices'
import {
	buildInvoiceHistory,
	creditNoteKindLabel,
	invoiceHeading,
	paymentTermsLine,
	savedInvoiceTotals,
} from './invoice-document'

function invoice(overrides: Partial<Invoice> = {}): Invoice {
	return {
		id: 'invoice-1',
		organization_id: 'org-1',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		kind: 'STANDARD',
		status: 'ISSUED',
		number: 'FAC-001',
		gross_cents: 12000,
		net_cents: 10000,
		lines: [],
		vat_breakdown: [{ rate_bp: 2000, vat_cents: 2000 }],
		issued_at: '2026-06-02T09:00:00Z',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}

const EMPTY_ACTIVITY = { sentAt: null, reminders: [] }

describe('invoiceHeading', () => {
	it('names each kind of document', () => {
		expect(invoiceHeading('STANDARD')).toBe('Facture')
		expect(invoiceHeading('DEPOSIT')).toBe('Facture d’acompte')
		expect(invoiceHeading('FINAL')).toBe('Facture de solde')
		expect(invoiceHeading('CREDIT_NOTE')).toBe('Avoir')
	})
})

describe('savedInvoiceTotals', () => {
	it('relays the totals the backend stored', () => {
		expect(savedInvoiceTotals(invoice())).toEqual({
			netCents: 10000,
			vat: [{ rateBp: 2000, vatCents: 2000 }],
			grossCents: 12000,
		})
	})
})

describe('buildInvoiceHistory', () => {
	it('tells the story of the invoice from what the backend knows', () => {
		const payment = {
			id: 'payment-1',
			amount_cents: 5000,
			method: 'Virement',
			created_at: '2026-06-05T09:00:00Z',
		} as InvoicePayment
		const entries = buildInvoiceHistory({
			invoice: invoice(),
			payments: [payment],
			creditNotes: [
				invoice({ id: 'credit-1', number: 'AV-001', kind: 'CREDIT_NOTE' }),
			],
			activity: {
				sentAt: '2026-06-03T09:00:00Z',
				reminders: ['2026-06-08T09:00:00Z'],
			},
		})

		expect(entries.map((entry) => entry.kind).sort()).toEqual(
			['created', 'credit_note', 'issued', 'note', 'payment', 'sent'].sort(),
		)
		expect(entries.find((entry) => entry.kind === 'issued')?.detail).toContain(
			'FAC-001',
		)
	})

	it('records a cancellation', () => {
		const entries = buildInvoiceHistory({
			invoice: invoice({ status: 'CANCELLED' }),
			payments: [],
			creditNotes: [],
			activity: EMPTY_ACTIVITY,
		})

		expect(entries.some((entry) => entry.detail === 'Facture annulée')).toBe(
			true,
		)
	})

	it('does not list an issue that never happened', () => {
		const entries = buildInvoiceHistory({
			invoice: invoice({ status: 'DRAFT', issued_at: null, number: null }),
			payments: [],
			creditNotes: [],
			activity: EMPTY_ACTIVITY,
		})

		expect(entries.map((entry) => entry.kind)).toEqual(['created'])
	})
})

describe('paymentTermsLine', () => {
	it('states payment on receipt when the delay is zero', () => {
		expect(
			paymentTermsLine('2026-06-02T09:00:00Z', '2026-06-02T00:00:00Z'),
		).toBe('Paiement à réception de la facture.')
	})

	it('states the delay in days otherwise', () => {
		expect(
			paymentTermsLine('2026-06-02T09:00:00Z', '2026-07-02T00:00:00Z'),
		).toBe('Délai de paiement : 30 jours.')
		expect(
			paymentTermsLine('2026-06-02T09:00:00Z', '2026-06-03T00:00:00Z'),
		).toBe('Délai de paiement : 1 jour.')
	})

	it('says nothing without a due date or when it precedes the issue', () => {
		expect(paymentTermsLine('2026-06-02T09:00:00Z', null)).toBeNull()
		expect(
			paymentTermsLine('2026-06-02T09:00:00Z', '2026-05-01T00:00:00Z'),
		).toBeNull()
	})
})

describe('refund history', () => {
	it('lists a mock refund among the payments', () => {
		const entries = buildInvoiceHistory({
			invoice: invoice(),
			payments: [],
			creditNotes: [],
			activity: EMPTY_ACTIVITY,
			refunds: [
				{
					id: 'refund-1',
					date: '2026-06-09',
					amountCents: 3000,
					method: 'Virement',
					reference: '',
					note: '',
				},
			],
		})

		expect(
			entries.find((entry) => entry.id === 'refund-refund-1')?.detail,
		).toContain('Remboursement')
	})
})

describe('creditNoteKindLabel', () => {
	it('is a total cancellation when it credits the whole invoice', () => {
		expect(
			creditNoteKindLabel({ gross_cents: 12000 }, { gross_cents: 12000 }),
		).toBe('Annulation totale')
		expect(
			creditNoteKindLabel({ gross_cents: 3000 }, { gross_cents: 12000 }),
		).toBe('Correction partielle')
		expect(creditNoteKindLabel({ gross_cents: 3000 }, undefined)).toBe(
			'Correction partielle',
		)
	})
})
