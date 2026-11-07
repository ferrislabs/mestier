import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import {
	EMPTY_TERMS,
	lineTermsAt,
	quoteLinkOf,
	termsOfQuote,
} from './quote-terms'

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'q1',
		organization_id: 'org-1',
		title: 'Salle de bain',
		status: 'SENT',
		customer_id: 'c1',
		customer_context_id: 'ctx1',
		lines: [],
		net_cents: 0,
		gross_cents: 0,
		vat_breakdown: [],
		deposit_percent: '0',
		is_expired: false,
		recurring: [],
		created_at: '2026-10-01T09:00:00Z',
		updated_at: '2026-10-01T09:00:00Z',
		...overrides,
	}
}

describe('termsOfQuote', () => {
	it('is empty for a quote that carries nothing', () => {
		expect(termsOfQuote(quote())).toEqual(EMPTY_TERMS)
	})

	it('reads every term from the API fields', () => {
		const terms = termsOfQuote(
			quote({
				valid_until: '2026-12-01',
				deposit_percent: '30.5',
				owner_member_id: 'm1',
				notes: 'Accès par la cour',
				responded_at: '2026-11-02T10:00:00Z',
				responded_by: 'Mme Durand',
				decline_reason: 'trop cher',
				deal_id: 'd1',
				revision_of_id: 'q0',
				superseded_by_id: 'q2',
			}),
		)

		expect(terms).toMatchObject({
			validUntil: '2026-12-01',
			depositPercent: 30.5,
			ownerId: 'm1',
			notes: 'Accès par la cour',
			respondedAt: '2026-11-02',
			signer: 'Mme Durand',
			declineReason: 'trop cher',
			dealId: 'd1',
			revisionOf: { id: 'q0' },
			supersededBy: { id: 'q2' },
		})
	})

	it('reads the discount and the frequency of each line', () => {
		const terms = termsOfQuote(
			quote({
				lines: [
					{
						id: 'l1',
						organization_id: 'org-1',
						quote_id: 'q1',
						label: 'Entretien',
						quantity: '1',
						unit: 'FLAT_RATE',
						unit_price_cents: 2000,
						discount_percent: '12.5',
						billing_frequency: 'QUARTERLY',
						net_cents: 1750,
						photo_keys: [],
						created_at: '2026-10-01T09:00:00Z',
						updated_at: '2026-10-01T09:00:00Z',
					},
				],
			}),
		)

		expect(lineTermsAt(terms, 0)).toEqual({
			discountPercent: 12.5,
			frequency: 'quarterly',
		})
		expect(lineTermsAt(terms, 3)).toEqual({
			discountPercent: 0,
			frequency: 'one_shot',
		})
	})

	it('labels the linked quotes once they are known', () => {
		const terms = termsOfQuote(quote({ revision_of_id: 'q0' }), {
			revisionOf: { id: 'q0', label: 'DEV-000' },
			supersededBy: null,
		})

		expect(terms.revisionOf).toEqual({ id: 'q0', label: 'DEV-000' })
	})
})

describe('quoteLinkOf', () => {
	it('names a quote by its reference, or calls it a draft', () => {
		expect(quoteLinkOf({ id: 'q0', reference: 'DEV-000' }, null)).toEqual({
			id: 'q0',
			label: 'DEV-000',
		})
		expect(quoteLinkOf({ id: 'q0', reference: null }, null)?.label).toBe(
			'Brouillon',
		)
		expect(quoteLinkOf(undefined, null)).toBeNull()
	})
})
