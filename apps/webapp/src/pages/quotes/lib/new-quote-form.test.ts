import { describe, expect, it } from 'vitest'
import { emptyQuoteLine, type QuoteFormValues } from '#/pages/quotes/types'
import { firstInvalidFieldId, validateNewQuote } from './new-quote-form'

function values(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: 'Salle de bain',
		customerId: 'customer-1',
		customerContextId: 'context-1',
		lines: [{ ...emptyQuoteLine(), label: 'Pose', unitPrice: '10' }],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

describe('validateNewQuote', () => {
	it('accepts a complete quote', () => {
		expect(validateNewQuote(values())).toEqual({})
	})

	it('asks for a customer before an address', () => {
		expect(
			validateNewQuote(values({ customerId: '', customerContextId: '' })),
		).toEqual({ customerId: 'Choisis un client.' })
	})

	it('asks for the billing address once a customer is chosen', () => {
		expect(validateNewQuote(values({ customerContextId: '' }))).toEqual({
			customerContextId: 'Choisis une adresse de facturation.',
		})
	})

	it('requires an object and valid lines', () => {
		const errors = validateNewQuote(
			values({ title: '  ', lines: [emptyQuoteLine()] }),
		)

		expect(errors.title).toBeDefined()
		expect(errors.lines).toBeDefined()
	})

	it('rejects a zero quantity', () => {
		const errors = validateNewQuote(
			values({
				lines: [
					{
						...emptyQuoteLine(),
						label: 'Pose',
						unitPrice: '10',
						quantity: '0',
					},
				],
			}),
		)

		expect(errors.lines).toBeDefined()
	})
})

describe('firstInvalidFieldId', () => {
	it('points at the first invalid field in form order', () => {
		expect(firstInvalidFieldId({ title: 'x', customerId: 'y' })).toBe(
			'new-quote-client',
		)
		expect(firstInvalidFieldId({})).toBeNull()
	})
})
