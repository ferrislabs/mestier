import { describe, expect, it } from 'vitest'
import { canInvoiceQuote, invoiceNewPath } from './quote-invoicing'

describe('invoiceNewPath', () => {
	it('leads to the new invoice form with the quote preselected', () => {
		expect(invoiceNewPath('atelier', 'q-1')).toBe(
			'/o/atelier/crm/factures/nouvelle?devis=q-1',
		)
	})
})

describe('canInvoiceQuote', () => {
	it('only lets an accepted quote be invoiced', () => {
		expect(canInvoiceQuote('accepted')).toBe(true)
		expect(canInvoiceQuote('sent')).toBe(false)
		expect(canInvoiceQuote('draft')).toBe(false)
		expect(canInvoiceQuote('superseded')).toBe(false)
	})
})
