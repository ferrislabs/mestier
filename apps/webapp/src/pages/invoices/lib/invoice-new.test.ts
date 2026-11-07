import { describe, expect, it } from 'vitest'
import type { QuoteInvoicingData } from '#/hooks/use-invoices'
import { invoiceFixture } from '#/pages/invoices/test-support/invoice-fixture'
import {
	defaultDepositPercent,
	depositPercentError,
	initialQuoteMode,
	MAX_PAYMENT_TERMS_DAYS,
	newInvoiceTermsError,
	parsePercent,
	parseTerms,
	quoteInvoicingView,
	readNewInvoiceSearch,
} from './invoice-new'

function data(overrides: Partial<QuoteInvoicingData> = {}): QuoteInvoicingData {
	return {
		invoiced_percent: '0',
		remaining_percent: '100',
		can_full: true,
		can_deposit: true,
		blocker: null,
		invoices: [],
		...overrides,
	}
}

describe('parsePercent', () => {
	it('reads a comma or a point and one decimal at most', () => {
		expect(parsePercent('30')).toBe(30)
		expect(parsePercent('12,5')).toBe(12.5)
		expect(parsePercent('12.5 %')).toBe(12.5)
		expect(parsePercent(' 40% ')).toBe(40)
		expect(parsePercent('12,55')).toBeNull()
		expect(parsePercent('-5')).toBeNull()
		expect(parsePercent('abc')).toBeNull()
		expect(parsePercent('')).toBeNull()
	})
})

describe('quoteInvoicingView', () => {
	it('relays what the backend decided on an uninvoiced quote', () => {
		expect(quoteInvoicingView(data())).toMatchObject({
			canFull: true,
			canDeposit: true,
			invoiced: 0,
			remaining: 100,
			reasons: [],
			existingInvoiceId: null,
		})
	})

	it('words the blocker of a quote that is not accepted', () => {
		const info = quoteInvoicingView(
			data({
				can_full: false,
				can_deposit: false,
				blocker: 'QUOTE_NOT_ACCEPTED',
			}),
		)

		expect(info.canFull).toBe(false)
		expect(info.canDeposit).toBe(false)
		expect(info.reasons).toEqual(['Le devis doit être accepté.'])
	})

	it('points at the existing full invoice', () => {
		const info = quoteInvoicingView(
			data({
				can_full: false,
				can_deposit: false,
				blocker: 'QUOTE_ALREADY_INVOICED',
				invoices: [invoiceFixture({ id: 'full-1', number: 'FAC-007' })],
			}),
		)

		expect(info.reasons).toEqual(['Ce devis a déjà une facture.'])
		expect(info.existingInvoiceId).toBe('full-1')
		expect(info.existingInvoiceNumber).toBe('FAC-007')
	})

	it('splits the deposits already invoiced into issued and still draft', () => {
		const info = quoteInvoicingView(
			data({
				invoiced_percent: '50',
				remaining_percent: '50',
				invoices: [
					invoiceFixture({ id: 'a', kind: 'DEPOSIT' }),
					invoiceFixture({
						id: 'b',
						kind: 'DEPOSIT',
						status: 'DRAFT',
						number: null,
					}),
				],
			}),
		)

		expect(info.invoiced).toBe(50)
		expect(info.remaining).toBe(50)
		expect(info.deposits.issued.map((item) => item.invoiceId)).toEqual(['a'])
		expect(info.deposits.drafts.map((item) => item.invoiceId)).toEqual(['b'])
		expect(info.existingInvoiceId).toBeNull()
	})

	it('leaves only the full invoice when deposits cover 100 %', () => {
		const info = quoteInvoicingView(
			data({
				invoiced_percent: '100',
				remaining_percent: '0',
				can_deposit: false,
				blocker: 'DEPOSITS_COVER_QUOTE',
			}),
		)

		expect(info.canDeposit).toBe(false)
		expect(info.canFull).toBe(true)
		expect(info.reasons[0]).toContain('100 %')
	})
})

describe('deposit percent', () => {
	it('defaults to 30 % without exceeding what remains', () => {
		expect(defaultDepositPercent(100)).toBe(30)
		expect(defaultDepositPercent(10)).toBe(10)
		expect(defaultDepositPercent(0)).toBe(0)
	})

	it('is capped at the remaining percentage', () => {
		expect(depositPercentError(30, '70')).toBeNull()
		expect(depositPercentError(30, '70,1')).toBe(
			'Le pourcentage dépasse le reste à facturer (70 %).',
		)
		expect(depositPercentError(0, '0')).toBe(
			'Le pourcentage doit être supérieur à zéro.',
		)
		expect(depositPercentError(0, 'x')).toBe(
			'Saisis un pourcentage valide, par exemple 30 ou 12,5.',
		)
	})
})

describe('payment terms', () => {
	it('reads a whole number of days', () => {
		expect(parseTerms('30')).toBe(30)
		expect(parseTerms(' 0 ')).toBe(0)
		expect(parseTerms('1,5')).toBeNull()
		expect(parseTerms('')).toBeNull()
	})

	it('allows 0 to 60 days', () => {
		expect(newInvoiceTermsError('0')).toBeNull()
		expect(newInvoiceTermsError(String(MAX_PAYMENT_TERMS_DAYS))).toBeNull()
		expect(newInvoiceTermsError('61')).toBe(
			'Le délai de paiement doit être compris entre 0 et 60 jours.',
		)
		expect(newInvoiceTermsError('abc')).toBe(
			'Le délai de paiement doit être compris entre 0 et 60 jours.',
		)
	})
})

describe('initialQuoteMode', () => {
	const both = { canFull: true, canDeposit: true, invoiced: 0 }

	it('follows the requested mode when it is available', () => {
		expect(initialQuoteMode(both, 'acompte')).toBe('deposit')
		expect(initialQuoteMode(both, 'complete')).toBe('full')
	})

	it('falls back to the full invoice, or to the deposit when only that remains', () => {
		expect(initialQuoteMode(both, undefined)).toBe('full')
		expect(
			initialQuoteMode(
				{ canFull: true, canDeposit: false, invoiced: 100 },
				'acompte',
			),
		).toBe('full')
		expect(
			initialQuoteMode(
				{ canFull: false, canDeposit: false, invoiced: 0 },
				undefined,
			),
		).toBe('deposit')
	})
})

describe('readNewInvoiceSearch', () => {
	it('reads the quote, the customer and the mode of the url', () => {
		expect(readNewInvoiceSearch('?devis=q1&client=c1&mode=acompte')).toEqual({
			quoteId: 'q1',
			clientId: 'c1',
			mode: 'acompte',
		})
	})

	it('ignores empty or unknown values', () => {
		expect(readNewInvoiceSearch('?devis=&mode=zzz')).toEqual({
			quoteId: undefined,
			clientId: undefined,
			mode: undefined,
		})
	})
})
