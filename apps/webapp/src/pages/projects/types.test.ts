import { describe, expect, it } from 'vitest'
import {
	canInvoiceAgainstQuote,
	depositPreviewCents,
	percentToBasisPoints,
} from '#/pages/projects/types'

describe('percentToBasisPoints', () => {
	it('reads a percent, comma or point, to the nearest basis point', () => {
		expect(percentToBasisPoints('30')).toBe(3000)
		expect(percentToBasisPoints('33,33')).toBe(3333)
		expect(percentToBasisPoints('33.33')).toBe(3333)
		expect(percentToBasisPoints('0,5')).toBe(50)
	})

	it('treats unusable or non-positive input as nothing rather than guessing', () => {
		expect(percentToBasisPoints('')).toBeNull()
		expect(percentToBasisPoints('abc')).toBeNull()
		expect(percentToBasisPoints('0')).toBeNull()
		expect(percentToBasisPoints('-10')).toBeNull()
	})
})

describe('depositPreviewCents', () => {
	it('matches a round percentage exactly', () => {
		// 30% of 1 000,00 € is 300,00 €.
		expect(depositPreviewCents(100_000, 3000)).toBe(30_000)
	})

	/**
	 * Mirrors `deposit_amount_cents`'s own `div_round_half_even` in
	 * `libs/core/src/domain/invoice/service.rs`: an exact half lands on the
	 * even neighbour, not always up.
	 */
	it('rounds an exact half to the even neighbour, same as the backend', () => {
		// 50% of 3,33 € is 1,665 € exactly: 166 is already even, stays 166.
		expect(depositPreviewCents(333, 5000)).toBe(166)
		// 50% of 3,31 € is 1,655 € exactly: 165 is odd, rounds up to 166.
		expect(depositPreviewCents(331, 5000)).toBe(166)
	})
})

describe('canInvoiceAgainstQuote', () => {
	it('needs a quote for a percentage or a remaining-balance invoice', () => {
		expect(canInvoiceAgainstQuote(null)).toBe(false)
		expect(canInvoiceAgainstQuote(420_000)).toBe(true)
		expect(canInvoiceAgainstQuote(0)).toBe(true)
	})
})
