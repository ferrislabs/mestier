import { describe, expect, it } from 'vitest'
import type { Quote } from '#/hooks/use-quotes'
import { EMPTY_QUOTE_EXTRAS } from './quote-extras'
import { buildQuoteSideInfo } from './quote-side-info'

const TODAY = '2026-11-07'

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'q1',
		organization_id: 'org-1',
		title: 'Salle de bain',
		status: 'SENT',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: 0,
		gross_cents: 0,
		vat_breakdown: [],
		created_at: '2026-10-01T09:00:00Z',
		updated_at: '2026-10-02T09:00:00Z',
		...overrides,
	}
}

const OWNERS = [{ id: 'm1', name: 'Julien Roche' }]

describe('buildQuoteSideInfo', () => {
	it('derives the display status and the validity note', () => {
		const info = buildQuoteSideInfo(
			quote(),
			{ ...EMPTY_QUOTE_EXTRAS, validUntil: '2026-11-01' },
			OWNERS,
			TODAY,
		)

		expect(info.displayStatus).toBe('expired')
		expect(info.validity.expired).toBe(true)
	})

	it('names the owner', () => {
		const info = buildQuoteSideInfo(
			quote(),
			{ ...EMPTY_QUOTE_EXTRAS, ownerId: 'm1' },
			OWNERS,
			TODAY,
		)

		expect(info.ownerName).toBe('Julien Roche')
	})

	it('has no owner name when the owner is unknown', () => {
		const info = buildQuoteSideInfo(
			quote(),
			{ ...EMPTY_QUOTE_EXTRAS, ownerId: 'gone' },
			OWNERS,
			TODAY,
		)

		expect(info.ownerName).toBeNull()
	})

	it('tells who accepted and when', () => {
		const info = buildQuoteSideInfo(
			quote({ status: 'ACCEPTED' }),
			{
				...EMPTY_QUOTE_EXTRAS,
				signer: 'Mme Durand',
				respondedAt: '2026-11-02',
			},
			OWNERS,
			TODAY,
		)

		expect(info.answerNote).toMatch(/^Accepté le 2 nov\. 2026 par Mme Durand$/)
	})

	it('tells when a quote was declined, with the reason', () => {
		const info = buildQuoteSideInfo(
			quote({ status: 'DECLINED' }),
			{
				...EMPTY_QUOTE_EXTRAS,
				respondedAt: '2026-11-02',
				declineReason: 'trop cher',
			},
			OWNERS,
			TODAY,
		)

		expect(info.answerNote).toMatch(/^Refusé le 2 nov\. 2026$/)
		expect(info.declineReason).toBe('trop cher')
	})

	it('falls back on the last update when no answer date was kept', () => {
		const info = buildQuoteSideInfo(
			quote({ status: 'ACCEPTED', updated_at: '2026-10-05T09:00:00Z' }),
			EMPTY_QUOTE_EXTRAS,
			OWNERS,
			TODAY,
		)

		expect(info.answerNote).toMatch(/^Accepté le 5 oct\. 2026$/)
	})

	it('has no answer note on a quote nobody answered', () => {
		expect(
			buildQuoteSideInfo(quote(), EMPTY_QUOTE_EXTRAS, OWNERS, TODAY).answerNote,
		).toBeNull()
	})

	it('relays the links of the revision chain and the opportunity', () => {
		const info = buildQuoteSideInfo(
			quote(),
			{
				...EMPTY_QUOTE_EXTRAS,
				revisionOf: { id: 'q0', label: 'DEV-000' },
				supersededBy: { id: 'q2', label: 'Brouillon' },
				dealId: 'd1',
			},
			OWNERS,
			TODAY,
		)

		expect(info.revisionOf).toEqual({ id: 'q0', label: 'DEV-000' })
		expect(info.supersededBy).toEqual({ id: 'q2', label: 'Brouillon' })
		expect(info.dealId).toBe('d1')
	})
})
