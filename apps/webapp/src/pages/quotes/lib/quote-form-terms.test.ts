import { describe, expect, it } from 'vitest'
import { emptyQuoteLine, type QuoteFormValues } from '#/pages/quotes/types'
import {
	areTermsValid,
	isPercentValid,
	lineTermsBody,
	lineWithTerms,
	percentText,
	termsBody,
	termsFormFields,
} from './quote-form-terms'
import { EMPTY_TERMS } from './quote-terms'

function form(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: 'Devis',
		customerId: 'c1',
		customerContextId: 'ctx1',
		lines: [emptyQuoteLine()],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

describe('percentText', () => {
	it('sends zero for an empty field and a dot for a comma', () => {
		expect(percentText('')).toBe('0')
		expect(percentText('  ')).toBe('0')
		expect(percentText('12,5')).toBe('12.5')
	})
})

describe('isPercentValid', () => {
	it('accepts an empty field, 0 to 100 and the decimals the backend stores', () => {
		expect(isPercentValid('', 1)).toBe(true)
		expect(isPercentValid('100', 1)).toBe(true)
		expect(isPercentValid('30,5', 1)).toBe(true)
		expect(isPercentValid('30,50', 1)).toBe(true)
		expect(isPercentValid('12,25', 2)).toBe(true)
	})

	it('refuses what the backend would refuse', () => {
		expect(isPercentValid('100,1', 1)).toBe(false)
		expect(isPercentValid('-1', 1)).toBe(false)
		expect(isPercentValid('30,55', 1)).toBe(false)
		expect(isPercentValid('12,255', 2)).toBe(false)
		expect(isPercentValid('abc', 1)).toBe(false)
	})
})

describe('areTermsValid', () => {
	it('checks the deposit and every discount', () => {
		expect(areTermsValid(form())).toBe(true)
		expect(areTermsValid(form({ depositPercent: '101' }))).toBe(false)
		expect(
			areTermsValid(
				form({ lines: [{ ...emptyQuoteLine(), discountPercent: '150' }] }),
			),
		).toBe(false)
	})
})

describe('termsBody', () => {
	it('builds the terms of the request without any total', () => {
		expect(
			termsBody(
				form({
					validUntil: '2026-12-01',
					depositPercent: '30',
					ownerId: 'm1',
					notes: ' Accès ',
				}),
			),
		).toEqual({
			valid_until: '2026-12-01',
			deposit_percent: '30',
			owner_member_id: 'm1',
			notes: 'Accès',
		})
	})

	it('sends nothing for an empty validity, owner or note', () => {
		expect(termsBody(form())).toEqual({
			valid_until: null,
			deposit_percent: '0',
			owner_member_id: null,
			notes: null,
		})
	})
})

describe('lineTermsBody', () => {
	it('sends the discount and the frequency of a line in the API vocabulary', () => {
		expect(
			lineTermsBody({
				...emptyQuoteLine(),
				discountPercent: '10',
				frequency: 'monthly',
			}),
		).toEqual({ discount_percent: '10', billing_frequency: 'MONTHLY' })
		expect(lineTermsBody(emptyQuoteLine())).toEqual({
			discount_percent: '0',
			billing_frequency: 'ONE_SHOT',
		})
	})
})

describe('the form round trip', () => {
	it('fills the form from the terms of a saved quote', () => {
		expect(
			termsFormFields({
				...EMPTY_TERMS,
				ownerId: 'm1',
				validUntil: '2026-12-01',
				depositPercent: 30,
				notes: 'Accès',
			}),
		).toEqual({
			ownerId: 'm1',
			validUntil: '2026-12-01',
			depositPercent: '30',
			notes: 'Accès',
		})
		expect(termsFormFields(EMPTY_TERMS).depositPercent).toBe('')
	})

	it('puts the discount and the frequency back on the line', () => {
		const line = lineWithTerms(
			emptyQuoteLine(),
			{ lines: [{ discountPercent: 10, frequency: 'yearly' }] },
			0,
		)

		expect(line).toMatchObject({ discountPercent: '10', frequency: 'yearly' })
	})
})
