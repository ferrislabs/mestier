import { describe, expect, it } from 'vitest'
import { emptyInvoiceLine } from '#/pages/invoices/types'
import {
	firstInvalidInvoiceFieldId,
	type NewInvoiceFormValues,
	validateNewInvoice,
} from './new-invoice-form'

function values(
	overrides: Partial<NewInvoiceFormValues> = {},
): NewInvoiceFormValues {
	return {
		title: '',
		legalNotice: '',
		termsDays: '30',
		kind: 'STANDARD',
		projectId: '',
		customerId: 'customer-1',
		customerContextId: 'context-1',
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

	it('keeps the payment delay between 0 and 60 days', () => {
		expect(validateNewInvoice(values({ termsDays: '0' }))).toEqual({})
		expect(validateNewInvoice(values({ termsDays: '60' }))).toEqual({})
		expect(validateNewInvoice(values({ termsDays: '61' })).terms).toContain(
			'entre 0 et 60 jours',
		)
		expect(validateNewInvoice(values({ termsDays: 'abc' })).terms).toBeDefined()
		expect(validateNewInvoice(values({ termsDays: '' }))).toEqual({})
	})

	it('needs at least one complete line', () => {
		expect(validateNewInvoice(values({ lines: [] })).lines).toBeDefined()
		expect(
			validateNewInvoice(values({ lines: [emptyInvoiceLine()] })).lines,
		).toBeDefined()
	})

	it('takes a negative price, a deduction line', () => {
		expect(
			validateNewInvoice(
				values({
					lines: [{ ...emptyInvoiceLine(), label: 'Remise', unitPrice: '-10' }],
				}),
			),
		).toEqual({})
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
