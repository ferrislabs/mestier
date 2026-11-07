import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	creditNoteRowsOf,
	invoiceRowsOf,
	paymentRowsOf,
} from '#/pages/customers/lib/activity'

function invoice(
	overrides: Partial<Schemas.InvoiceResponse>,
): Schemas.InvoiceResponse {
	return {
		id: 'i1',
		customer_id: 'c1',
		kind: 'STANDARD',
		status: 'ISSUED',
		number: 'F-001',
		gross_cents: 120_00,
		net_cents: 100_00,
		issued_at: '2026-10-01T00:00:00Z',
		created_at: '2026-09-30T00:00:00Z',
		...overrides,
	} as Schemas.InvoiceResponse
}

function payment(
	overrides: Partial<Schemas.InvoicePaymentResponse>,
): Schemas.InvoicePaymentResponse {
	return {
		id: 'p1',
		invoice_id: 'i1',
		amount_cents: 50_00,
		method: 'Virement',
		paid_on: '2026-10-10',
		created_at: '2026-10-10T08:00:00Z',
		...overrides,
	} as Schemas.InvoicePaymentResponse
}

describe('invoiceRowsOf', () => {
	it('keeps the invoices of the customer, without credit notes', () => {
		const rows = invoiceRowsOf(
			[
				invoice({ id: 'a' }),
				invoice({ id: 'b', kind: 'CREDIT_NOTE' }),
				invoice({ id: 'c', customer_id: 'c2' }),
			],
			'c1',
		)
		expect(rows.map((row) => row.id)).toEqual(['a'])
	})
})

describe('creditNoteRowsOf', () => {
	const source = invoice({ id: 'src', number: 'F-007', gross_cents: 120_00 })

	it('lists the credit notes of the customer, newest first, with their source', () => {
		const rows = creditNoteRowsOf(
			[
				source,
				invoice({
					id: 'n1',
					kind: 'CREDIT_NOTE',
					number: 'AV-001',
					source_invoice_id: 'src',
					gross_cents: 120_00,
					issued_at: '2026-10-05T00:00:00Z',
				}),
				invoice({
					id: 'n2',
					kind: 'CREDIT_NOTE',
					number: 'AV-002',
					source_invoice_id: 'src',
					gross_cents: 20_00,
					issued_at: '2026-10-20T00:00:00Z',
				}),
				invoice({ id: 'n3', kind: 'CREDIT_NOTE', customer_id: 'c2' }),
			],
			'c1',
		)
		expect(rows.map((row) => row.id)).toEqual(['n2', 'n1'])
		expect(rows[0]).toMatchObject({
			label: 'AV-002',
			sourceLabel: 'F-007',
			kind: 'Avoir partiel',
			total: 20_00,
		})
		expect(rows[1].kind).toBe('Annulation totale')
	})
})

describe('paymentRowsOf', () => {
	it('lists the live payments newest first, labelled by their invoice', () => {
		const rows = paymentRowsOf([
			{
				invoice: invoice({ id: 'i1', number: 'F-001' }),
				payments: [
					payment({ id: 'p1', paid_on: '2026-10-10' }),
					payment({ id: 'p2', paid_on: '2026-10-12', reference: 'VIR-9' }),
					payment({ id: 'p3', deleted_at: '2026-10-13T00:00:00Z' }),
				],
			},
		])
		expect(rows.map((row) => row.id)).toEqual(['p2', 'p1'])
		expect(rows[0]).toMatchObject({
			invoiceId: 'i1',
			target: 'F-001',
			detail: 'Virement · VIR-9',
			amount: 50_00,
			date: '2026-10-12',
		})
		expect(rows[1].detail).toBe('Virement')
	})
})
