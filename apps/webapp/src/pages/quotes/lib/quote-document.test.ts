import { describe, expect, it } from 'vitest'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import { emptyQuoteLine, type QuoteFormValues } from '#/pages/quotes/types'
import {
	buildQuoteHistory,
	buildQuotePaper,
	previewQuoteTotals,
	savedQuoteTotals,
} from './quote-document'
import { EMPTY_TERMS } from './quote-terms'

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'quote-1',
		organization_id: 'org-1',
		title: 'Salle de bain',
		status: 'DRAFT',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: 10000,
		gross_cents: 12000,
		vat_breakdown: [{ rate_bp: 2000, vat_cents: 2000 }],
		deposit_percent: '0',
		is_expired: false,
		recurring: [],
		created_at: '2026-01-01T09:00:00Z',
		updated_at: '2026-01-01T09:00:00Z',
		...overrides,
	}
}

describe('savedQuoteTotals', () => {
	it('relays the totals the backend stored', () => {
		expect(savedQuoteTotals(quote())).toEqual({
			netCents: 10000,
			vat: [{ rateBp: 2000, vatCents: 2000 }],
			grossCents: 12000,
			recurring: [],
		})
	})

	it('relays the recurring totals of the backend, one per frequency', () => {
		const totals = savedQuoteTotals(
			quote({
				recurring: [
					{
						frequency: 'MONTHLY',
						net_cents: 2000,
						vat_cents: 400,
						gross_cents: 2400,
					},
				],
			}),
		)

		expect(totals.recurring).toEqual([
			{ frequency: 'monthly', netCents: 2000, vatCents: 400, grossCents: 2400 },
		])
	})
})

function form(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: '',
		customerId: '',
		customerContextId: '',
		lines: [],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

describe('previewQuoteTotals', () => {
	it('has no VAT line when the organization is not subject to VAT', () => {
		const totals = previewQuoteTotals(
			form({
				lines: [{ ...emptyQuoteLine(), quantity: '2', unitPrice: '10' }],
			}),
			false,
		)

		expect(totals).toEqual({ netCents: 2000, vat: [], grossCents: 2000 })
	})

	it('adds the VAT of each rate when the organization is subject', () => {
		const totals = previewQuoteTotals(
			form({
				lines: [
					{
						...emptyQuoteLine(),
						quantity: '1',
						unitPrice: '100',
						vatRateBp: '2000',
					},
				],
			}),
			true,
		)

		expect(totals.grossCents).toBe(12000)
		expect(totals.vat).toEqual([{ rateBp: 2000, vatCents: 2000 }])
	})
})

describe('buildQuoteHistory', () => {
	it('only lists the creation of a quote never touched', () => {
		expect(
			buildQuoteHistory(quote(), EMPTY_TERMS).map((entry) => entry.kind),
		).toEqual(['created'])
	})

	it('reads the last status change as an event', () => {
		const entries = buildQuoteHistory(
			quote({ status: 'ACCEPTED', updated_at: '2026-01-05T09:00:00Z' }),
			EMPTY_TERMS,
		)

		expect(entries.map((entry) => entry.kind)).toEqual(['created', 'accepted'])
	})

	it('names the signer of an acceptance', () => {
		const entries = buildQuoteHistory(
			quote({ status: 'ACCEPTED', updated_at: '2026-01-05T09:00:00Z' }),
			{ ...EMPTY_TERMS, signer: 'Mme Durand' },
		)

		expect(entries.at(-1)?.detail).toBe('Devis accepté par Mme Durand')
	})

	it('gives the reason of a refusal', () => {
		const entries = buildQuoteHistory(
			quote({ status: 'DECLINED', updated_at: '2026-01-05T09:00:00Z' }),
			{ ...EMPTY_TERMS, declineReason: 'trop cher' },
		)

		expect(entries.at(-1)?.detail).toBe('Devis refusé : trop cher')
	})

	it('shows the quote this one revises', () => {
		const entries = buildQuoteHistory(quote(), {
			...EMPTY_TERMS,
			revisionOf: { id: 'q0', label: 'DEV-000' },
		})

		expect(entries.map((entry) => entry.kind)).toEqual(['created', 'revised'])
		expect(entries[1]?.detail).toBe('Révision de DEV-000')
	})

	it('says by which quote a replaced quote was succeeded', () => {
		const entries = buildQuoteHistory(
			quote({ status: 'SUPERSEDED', updated_at: '2026-01-04T09:00:00Z' }),
			{ ...EMPTY_TERMS, supersededBy: { id: 'q2', label: 'DEV-002' } },
		)

		expect(entries.map((entry) => entry.kind)).toEqual(['created', 'revised'])
		expect(entries[1]?.detail).toBe('Remplacé par DEV-002')
	})
})

describe('buildQuotePaper', () => {
	const organization = {
		id: 'org-1',
		name: 'Atelier',
		vat_status: { type: 'subject', vat_number: 'FR1' },
	} as unknown as Organization

	function paperOf(values: QuoteFormValues) {
		return buildQuotePaper({
			quote: quote(),
			values,
			customer: undefined,
			context: {
				id: 'context-1',
				customer_id: 'customer-1',
				label: 'Chantier Lyon',
				address_line: '5 rue des Forges',
				postal_code: '69001',
				city: 'Lyon',
				country: 'FR',
				kind: 'SITE',
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
			organization,
			totals: { netCents: 0, vat: [], grossCents: 0 },
		})
	}

	it('prints the validity date and the site address', () => {
		const paper = paperOf(form({ validUntil: '2026-12-01' }))

		expect(paper.validUntil).toBe('2026-12-01')
		expect(paper.siteAddressLines).toEqual([
			'Chantier Lyon',
			'5 rue des Forges',
			'69001 Lyon',
		])
	})

	it('has no validity date until one is chosen', () => {
		expect(paperOf(form()).validUntil).toBeNull()
	})

	it('carries the discount and the frequency of each line', () => {
		const paper = paperOf(
			form({
				lines: [
					{
						...emptyQuoteLine(),
						label: 'Maintenance',
						unitPrice: '10',
						discountPercent: '10',
						frequency: 'monthly',
					},
				],
			}),
		)

		expect(paper.lines[0]).toMatchObject({
			discountPercent: 10,
			frequency: 'monthly',
		})
	})

	it('prints the notes of the quote', () => {
		expect(paperOf(form({ notes: 'Acompte à la commande' })).notes).toBe(
			'Acompte à la commande',
		)
	})
})
