import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import {
	activeQuoteLinks,
	defaultDepositPercent,
	depositLines,
	depositPercentError,
	depositPreviewCents,
	fullInvoiceLines,
	initialQuoteMode,
	MAX_PAYMENT_TERMS_DAYS,
	newInvoiceTermsError,
	parsePercent,
	parseTerms,
	paymentTermsDueDate,
	type QuoteInvoiceLink,
	quoteInvoicing,
	readNewInvoiceSearch,
} from './invoice-new'

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'quote-1',
		reference: 'D-2026-001',
		status: 'ACCEPTED',
		lines: [
			{
				id: 'l1',
				label: 'Pose',
				quantity: '1',
				unit_price_cents: 100000,
				vat_rate_bp: 2000,
			},
			{
				id: 'l2',
				label: 'Fournitures',
				quantity: '2',
				unit_price_cents: 50000,
				vat_rate_bp: 1000,
			},
		],
		...overrides,
	} as unknown as Quote
}

function link(overrides: Partial<QuoteInvoiceLink>): QuoteInvoiceLink {
	return {
		invoiceId: 'inv-1',
		kind: 'deposit',
		percent: 30,
		status: 'ISSUED',
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

describe('quoteInvoicing', () => {
	it('offers both modes on an accepted, uninvoiced quote', () => {
		const info = quoteInvoicing(quote(), [])
		expect(info).toMatchObject({
			canFull: true,
			canDeposit: true,
			invoiced: 0,
			remaining: 100,
			reasons: [],
			existingInvoiceId: null,
		})
	})

	it('refuses a quote that is not accepted', () => {
		const info = quoteInvoicing(quote({ status: 'SENT' }), [])
		expect(info.canFull).toBe(false)
		expect(info.canDeposit).toBe(false)
		expect(info.reasons).toEqual(['Le devis doit être accepté.'])
	})

	it('refuses both modes once the quote has a full invoice', () => {
		const info = quoteInvoicing(quote(), [
			link({ invoiceId: 'full-1', kind: 'full', percent: 100 }),
		])
		expect(info.canFull).toBe(false)
		expect(info.canDeposit).toBe(false)
		expect(info.reasons).toEqual(['Ce devis a déjà une facture.'])
		expect(info.existingInvoiceId).toBe('full-1')
	})

	it('ignores cancelled invoices', () => {
		const info = quoteInvoicing(quote(), [
			link({ kind: 'full', percent: 100, status: 'CANCELLED' }),
			link({ invoiceId: 'inv-2', status: 'CANCELLED' }),
		])
		expect(info.canFull).toBe(true)
		expect(info.invoiced).toBe(0)
	})

	it('tracks the deposits already invoiced, issued or still draft', () => {
		const info = quoteInvoicing(quote(), [
			link({ invoiceId: 'a', percent: 30 }),
			link({ invoiceId: 'b', percent: 20, status: 'DRAFT' }),
		])
		expect(info.invoiced).toBe(50)
		expect(info.remaining).toBe(50)
		expect(info.deposits.issued.map((item) => item.invoiceId)).toEqual(['a'])
		expect(info.deposits.drafts.map((item) => item.invoiceId)).toEqual(['b'])
	})

	it('leaves only the full invoice when deposits cover 100 %', () => {
		const info = quoteInvoicing(quote(), [link({ percent: 100 })])
		expect(info.remaining).toBe(0)
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

describe('deposit and full lines', () => {
	it('splits a deposit by VAT rate, each base rounded', () => {
		const lines = depositLines(quote(), 30)
		expect(lines).toEqual([
			{
				label: 'Acompte de 30 % sur devis D-2026-001 (TVA 20 %)',
				quantity: '1',
				unit_price_cents: 30000,
				vat_rate_basis_points: 2000,
			},
			{
				label: 'Acompte de 30 % sur devis D-2026-001 (TVA 10 %)',
				quantity: '1',
				unit_price_cents: 30000,
				vat_rate_basis_points: 1000,
			},
		])
	})

	it('does not mention the rate when the quote has a single one', () => {
		const single = quote({
			lines: [
				{
					id: 'l1',
					label: 'Pose',
					quantity: '1',
					unit_price_cents: 100000,
					vat_rate_bp: 2000,
				},
			],
		} as unknown as Partial<Quote>)
		expect(depositLines(single, 50)[0]?.label).toBe(
			'Acompte de 50 % sur devis D-2026-001',
		)
	})

	it('previews the deposit with VAT', () => {
		expect(depositPreviewCents(quote(), 30)).toBe(69000)
		expect(depositPreviewCents(quote(), 0)).toBe(0)
	})

	it('copies the quote lines for a full invoice', () => {
		expect(fullInvoiceLines(quote())).toEqual([
			{
				label: 'Pose',
				quantity: '1',
				unit_price_cents: 100000,
				vat_rate_basis_points: 2000,
			},
			{
				label: 'Fournitures',
				quantity: '2',
				unit_price_cents: 50000,
				vat_rate_basis_points: 1000,
			},
		])
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

	it('turns the delay into a due day', () => {
		expect(paymentTermsDueDate('2026-06-10', '30')).toBe('2026-07-10')
		expect(paymentTermsDueDate('2026-06-10', '0')).toBe('2026-06-10')
		expect(paymentTermsDueDate('2026-06-10', 'x')).toBeNull()
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

describe('activeQuoteLinks', () => {
	const links = {
		a: { quoteId: 'q-1', kind: 'deposit' as const, percent: 30 },
		b: { quoteId: 'q-2', kind: 'full' as const, percent: 100 },
		c: { quoteId: 'q-1', kind: 'deposit' as const, percent: 20 },
	}

	it('keeps the links of the quote, drops hidden drafts and reads the live status', () => {
		expect(
			activeQuoteLinks({
				quoteId: 'q-1',
				links,
				hiddenIds: ['c'],
				statusById: new Map([['a', 'PAID' as const]]),
			}),
		).toEqual([
			{ invoiceId: 'a', kind: 'deposit', percent: 30, status: 'PAID' },
		])
	})

	it('treats an invoice missing from the loaded page as issued', () => {
		expect(
			activeQuoteLinks({
				quoteId: 'q-2',
				links,
				hiddenIds: [],
				statusById: new Map(),
			})[0]?.status,
		).toBe('ISSUED')
	})
})
