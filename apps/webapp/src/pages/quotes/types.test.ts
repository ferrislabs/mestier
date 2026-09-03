import { describe, expect, it } from 'vitest'
import {
	eurosToCents,
	formatVatRateBp,
	type QuoteLineFormValues,
	quoteLinesGrossCents,
	quoteLinesVatBreakdown,
	quoteLineTotalCents,
	quoteReferenceLabel,
} from '#/pages/quotes/types'

function line(quantity: string, unitPrice: string): QuoteLineFormValues {
	return {
		clientId: 'line-1',
		catalogItemId: '',
		catalogItemType: 'CUSTOM',
		serviceRateId: '',
		label: 'Taille de haie',
		quantity,
		unit: 'HOUR',
		unitPrice,
		vatRateBp: '',
		notes: '',
		photoKeys: [],
	}
}

describe('eurosToCents', () => {
	it('reads what an artisan types, comma or point', () => {
		expect(eurosToCents('45,00')).toBe(4500)
		expect(eurosToCents('45.00')).toBe(4500)
		expect(eurosToCents('118,45')).toBe(11845)
		expect(eurosToCents('0,05')).toBe(5)
		expect(eurosToCents('1000')).toBe(100000)
	})

	/**
	 * A third decimal on a euro amount has no backend counterpart to match, so
	 * it follows the same rule as the rest of this file and lands on the even
	 * cent. What matters is that it is exact: `1.005 * 100` in floating point is
	 * `100.49999999999999`, which is not a half at all.
	 */
	it('rounds a third decimal to the even cent, exactly', () => {
		expect(eurosToCents('1,005')).toBe(100)
		expect(eurosToCents('1,015')).toBe(102)
		expect(eurosToCents('2,675')).toBe(268)
	})

	it('treats unusable input as nothing rather than guessing', () => {
		expect(eurosToCents('')).toBe(0)
		expect(eurosToCents('   ')).toBe(0)
		expect(eurosToCents('abc')).toBe(0)
		expect(eurosToCents('.')).toBe(0)
	})
})

describe('quoteLineTotalCents', () => {
	it('multiplies a quantity by a unit price', () => {
		expect(quoteLineTotalCents(line('4', '45,00'))).toBe(18000)
		expect(quoteLineTotalCents(line('12,5', '38,00'))).toBe(47500)
		expect(quoteLineTotalCents(line('8', '27,50'))).toBe(22000)
	})

	/**
	 * The same vectors `domain::quote::service` pins, and the reason both files
	 * carry them: the draft total is computed here while the stored total is
	 * computed in Rust, so one pricing rule has two implementations. Each product
	 * below lands exactly on a half cent, which is where the rounding mode
	 * decides the answer, and each goes to the even neighbour rather than up.
	 *
	 * These are also the cases the previous float implementation got wrong, by
	 * one cent, roughly once every fifty lines.
	 */
	it('agrees with the backend on products that land on a half cent', () => {
		expect(quoteLineTotalCents(line('631,9', '388,75'))).toBe(24_565_112)
		expect(quoteLineTotalCents(line('1723,1', '54,55'))).toBe(9_399_510)
		expect(quoteLineTotalCents(line('710,1', '134,85'))).toBe(9_575_698)
		expect(quoteLineTotalCents(line('1274,7', '138,95'))).toBe(17_711_956)
		expect(quoteLineTotalCents(line('1024,5', '375,25'))).toBe(38_444_362)
		expect(quoteLineTotalCents(line('1744,9', '263,85'))).toBe(46_039_186)
	})

	/** Verified against the backend: 0,5 goes to 0 and 1,5 goes to 2. */
	it('rounds a half to its even neighbour, in both directions', () => {
		expect(quoteLineTotalCents(line('0,5', '0,01'))).toBe(0)
		expect(quoteLineTotalCents(line('1,5', '0,01'))).toBe(2)
		expect(quoteLineTotalCents(line('2,5', '0,01'))).toBe(2)
		expect(quoteLineTotalCents(line('3,5', '0,01'))).toBe(4)
	})

	it('is nothing at all when the line has no quantity yet', () => {
		expect(quoteLineTotalCents(line('', '45,00'))).toBe(0)
		expect(quoteLineTotalCents(line('0', '45,00'))).toBe(0)
	})

	it('is nothing when the price has not been entered', () => {
		expect(quoteLineTotalCents(line('4', ''))).toBe(0)
	})
})

function lineWithVat(
	quantity: string,
	unitPrice: string,
	vatRateBp: string,
): QuoteLineFormValues {
	return { ...line(quantity, unitPrice), vatRateBp }
}

describe('quoteLinesVatBreakdown', () => {
	it('is empty for an organization not subject to VAT', () => {
		expect(
			quoteLinesVatBreakdown([lineWithVat('1', '100,00', '2000')], false),
		).toEqual([])
	})

	it('groups VAT per rate, sorted ascending', () => {
		expect(
			quoteLinesVatBreakdown(
				[
					lineWithVat('1', '100,00', '2000'),
					lineWithVat('1', '50,00', '1000'),
					lineWithVat('1', '20,00', '2000'),
				],
				true,
			),
		).toEqual([
			{ rateBp: 1000, vatCents: 500 },
			{ rateBp: 2000, vatCents: 2400 },
		])
	})

	/**
	 * The exact vector `domain::quote::service`'s
	 * `three_half_cent_vat_lines_round_per_line_then_sum` pins: three lines at
	 * 1,05 € net and 10 % VAT each round to 10 cents of VAT (10,5 lands on the
	 * even neighbour), summing to 30 — not 32, which is what rounding the
	 * 3,15 € net total once would give.
	 */
	it('rounds VAT per line before summing, agreeing with the backend', () => {
		const lines = [
			lineWithVat('1', '1,05', '1000'),
			lineWithVat('1', '1,05', '1000'),
			lineWithVat('1', '1,05', '1000'),
		]
		expect(quoteLinesVatBreakdown(lines, true)).toEqual([
			{ rateBp: 1000, vatCents: 30 },
		])
	})

	it('treats an unset rate as 0 %, not as excluded', () => {
		expect(
			quoteLinesVatBreakdown([lineWithVat('1', '100,00', '')], true),
		).toEqual([{ rateBp: 0, vatCents: 0 }])
	})

	it('leaves out lines with nothing chargeable yet', () => {
		expect(
			quoteLinesVatBreakdown([lineWithVat('', '100,00', '2000')], true),
		).toEqual([])
	})
})

describe('quoteLinesGrossCents', () => {
	it('adds the VAT breakdown on top of the net total', () => {
		expect(
			quoteLinesGrossCents(31500, [
				{ rateBp: 1000, vatCents: 30 },
				{ rateBp: 2000, vatCents: 2400 },
			]),
		).toBe(33930)
	})

	it('is the net total when there is no VAT to add', () => {
		expect(quoteLinesGrossCents(31500, [])).toBe(31500)
	})
})

describe('formatVatRateBp', () => {
	it('reads basis points as a percentage', () => {
		expect(formatVatRateBp(2000)).toBe('20 %')
		expect(formatVatRateBp(1000)).toBe('10 %')
		expect(formatVatRateBp(0)).toBe('0 %')
	})

	it('keeps the fractional part for a rate that has one', () => {
		expect(formatVatRateBp(550)).toBe('5.50 %')
		expect(formatVatRateBp(210)).toBe('2.10 %')
	})
})

describe('quoteReferenceLabel', () => {
	it('reads the allocated reference as-is', () => {
		expect(quoteReferenceLabel('DEV-2026-0001')).toBe('DEV-2026-0001')
	})

	/** #313: a draft has no number until it is sent — `null` here is a fact,
	 * not a missing value, and the label says so rather than showing a
	 * blank. */
	it('explains a draft rather than showing a blank', () => {
		expect(quoteReferenceLabel(null)).toBe('Numéro attribué à l’envoi')
		expect(quoteReferenceLabel(undefined)).toBe('Numéro attribué à l’envoi')
	})
})
