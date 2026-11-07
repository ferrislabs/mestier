import { describe, expect, it } from 'vitest'
import type { Invoice, InvoicePayment } from '#/hooks/use-invoices'
import {
	buildInvoiceHistory,
	creditNoteKindLabel,
	invoiceActivity,
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
		issued_on: '2026-06-02',
		due_on: '2026-07-02',
		title: 'Salle de bain',
		payment_terms_days: 30,
		legal_notice: 'Pénalités de retard : trois fois le taux légal.',
		sent_at: null,
		internal_note: null,
		full_cancellation: false,
		events: [],
		paid_cents: 0,
		credited_cents: 0,
		balance_cents: 12000,
		display_state: 'UNPAID',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}

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
			method: 'TRANSFER',
			created_at: '2026-06-05T09:00:00Z',
		} as InvoicePayment
		const entries = buildInvoiceHistory({
			invoice: invoice({
				events: [
					{
						id: 'event-1',
						kind: 'SENT',
						detail: 'Envoyée au client',
						at: '2026-06-03T09:00:00Z',
					},
					{
						id: 'event-2',
						kind: 'REMINDER',
						detail: 'Rappel envoyé',
						at: '2026-06-08T09:00:00Z',
					},
				],
			}),
			payments: [payment],
			creditNotes: [
				invoice({ id: 'credit-1', number: 'AV-001', kind: 'CREDIT_NOTE' }),
			],
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
	it('lists a refund, a negative payment, among the payments', () => {
		const entries = buildInvoiceHistory({
			invoice: invoice(),
			payments: [
				{
					id: 'refund-1',
					amount_cents: -3000,
					method: 'TRANSFER',
					created_at: '2026-06-09T09:00:00Z',
				} as InvoicePayment,
			],
			creditNotes: [],
		})

		expect(
			entries.find((entry) => entry.id === 'payment-refund-1')?.detail,
		).toContain('Remboursement')
	})
})

describe('invoiceActivity', () => {
	it('reads the send date, the reminders and the internal note off the invoice', () => {
		expect(
			invoiceActivity(
				invoice({
					sent_at: '2026-06-03T09:00:00Z',
					internal_note: 'Rappeler mardi',
					events: [
						{
							id: 'event-1',
							kind: 'REMINDER',
							detail: 'Rappel envoyé',
							at: '2026-06-08T09:00:00Z',
						},
						{
							id: 'event-2',
							kind: 'NOTE',
							detail: 'Rappeler mardi',
							at: '2026-06-09T09:00:00Z',
						},
					],
				}),
			),
		).toEqual({
			sentAt: '2026-06-03T09:00:00Z',
			reminders: ['2026-06-08T09:00:00Z'],
			internalNote: 'Rappeler mardi',
		})
	})
})

describe('creditNoteKindLabel', () => {
	it('is a total cancellation when the server flagged it so', () => {
		expect(creditNoteKindLabel({ full_cancellation: true })).toBe(
			'Annulation totale',
		)
		expect(creditNoteKindLabel({ full_cancellation: false })).toBe(
			'Correction partielle',
		)
	})
})
