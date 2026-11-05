import { describe, expect, it } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import {
	creditNoteRows,
	creditNotesSummary,
	filterCreditNoteRows,
	kindsLabel,
} from './credit-notes-list'

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
		vat_breakdown: [],
		issued_at: '2026-06-02T09:00:00Z',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}

const CUSTOMER = {
	id: 'customer-1',
	name: 'Menuiserie Dupont',
} as Customer

const FULL = invoice({
	id: 'credit-1',
	kind: 'CREDIT_NOTE',
	number: 'AV-001',
	source_invoice_id: 'invoice-1',
	gross_cents: 12000,
	notes: 'Annulation de chantier',
	issued_at: '2026-06-10T09:00:00Z',
})
const PARTIAL = invoice({
	id: 'credit-2',
	kind: 'CREDIT_NOTE',
	number: 'AV-002',
	source_invoice_id: 'invoice-1',
	gross_cents: 3000,
	notes: 'Taux de TVA erroné',
	issued_at: '2026-06-12T09:00:00Z',
})

describe('creditNoteRows', () => {
	it('joins each credit note to its invoice and customer, newest first', () => {
		const rows = creditNoteRows([FULL, PARTIAL], [invoice()], [CUSTOMER])

		expect(rows.map((row) => row.note.number)).toEqual(['AV-002', 'AV-001'])
		expect(rows[0]).toMatchObject({
			sourceLabel: 'FAC-001',
			customerName: 'Menuiserie Dupont',
			kindLabel: 'Correction partielle',
			totalCents: 3000,
		})
		expect(rows[1]?.kindLabel).toBe('Annulation totale')
	})

	it('keeps a credit note whose invoice is not loaded', () => {
		const rows = creditNoteRows([FULL], [], [CUSTOMER])

		expect(rows[0]?.sourceLabel).toBe('—')
		expect(rows[0]?.kindLabel).toBe('Correction partielle')
	})
})

describe('filterCreditNoteRows', () => {
	const rows = creditNoteRows([FULL, PARTIAL], [invoice()], [CUSTOMER])

	it('finds a credit note by number, invoice, customer or reason', () => {
		expect(filterCreditNoteRows(rows, 'av-002')).toHaveLength(1)
		expect(filterCreditNoteRows(rows, 'fac-001')).toHaveLength(2)
		expect(filterCreditNoteRows(rows, 'dupont')).toHaveLength(2)
		expect(filterCreditNoteRows(rows, 'tva errone')).toHaveLength(1)
	})

	it('returns everything without a query', () => {
		expect(filterCreditNoteRows(rows, '  ')).toHaveLength(2)
	})
})

describe('creditNotesSummary', () => {
	it('counts both kinds and adds the credited totals', () => {
		const summary = creditNotesSummary(
			creditNoteRows([FULL, PARTIAL], [invoice()], [CUSTOMER]),
		)

		expect(summary).toEqual({
			count: 2,
			totalCents: 15000,
			cancellations: 1,
			corrections: 1,
		})
		expect(kindsLabel(summary)).toBe(
			'1 annulation totale, 1 correction partielle',
		)
	})

	it('says so when there is none', () => {
		expect(kindsLabel({ cancellations: 0, corrections: 0 })).toBe(
			'Aucun avoir émis',
		)
	})
})
