import { describe, expect, it } from 'vitest'
import {
	emptyInvoiceLine,
	type InvoiceFormValues,
} from '#/pages/invoices/types'
import {
	firstInvalidInvoiceFieldId,
	validateNewInvoice,
} from './new-invoice-form'

function values(overrides: Partial<InvoiceFormValues> = {}): InvoiceFormValues {
	return {
		kind: 'STANDARD',
		projectId: '',
		customerId: 'customer-1',
		customerContextId: 'context-1',
		dueAt: '',
		notes: '',
		operationNature: '',
		lines: [{ ...emptyInvoiceLine(), label: 'Pose', unitPrice: '10' }],
		...overrides,
	}
}

describe('validateNewInvoice', () => {
	it('accepts a complete invoice', () => {
		expect(validateNewInvoice(values())).toEqual({})
	})

	it('asks for a customer, then for an address', () => {
		expect(
			validateNewInvoice(values({ customerId: '', customerContextId: '' })),
		).toEqual({ customerId: 'Choisis un client.' })
		expect(validateNewInvoice(values({ customerContextId: '' }))).toEqual({
			customerContextId: 'Choisis une adresse de facturation.',
		})
	})

	it('needs at least one complete line', () => {
		expect(validateNewInvoice(values({ lines: [] })).lines).toBeDefined()
		expect(
			validateNewInvoice(values({ lines: [emptyInvoiceLine()] })).lines,
		).toBeDefined()
	})

	it('refuses a negative price: the backend only takes positive unit prices', () => {
		expect(
			validateNewInvoice(
				values({
					lines: [{ ...emptyInvoiceLine(), label: 'Remise', unitPrice: '-10' }],
				}),
			).lines,
		).toBeDefined()
	})
})

describe('firstInvalidInvoiceFieldId', () => {
	it('points at the first invalid field in form order', () => {
		expect(firstInvalidInvoiceFieldId({ lines: 'x', customerId: 'y' })).toBe(
			'new-invoice-client',
		)
		expect(firstInvalidInvoiceFieldId({})).toBeNull()
	})
})
