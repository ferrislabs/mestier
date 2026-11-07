import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { ApiDeal } from '#/hooks/use-deals'
import {
	apiOfStage,
	dealBodyOfDraft,
	dealOfApi,
	dealQuoteOfApi,
	stageOfApi,
	summaryOfApi,
} from '#/pages/deals/lib/deal-api'
import { STAGES } from '#/pages/deals/lib/pipeline'

const API_DEAL: ApiDeal = {
	id: 'd1',
	organization_id: 'org',
	customer_id: 'c1',
	title: 'Fournil',
	amount_cents: 125000,
	stage: 'LOST',
	expected_close_date: '2026-12-15',
	owner_member_id: 'm1',
	source: 'salon',
	lost_reason: 'Prix trop élevé',
	notes: null,
	quote_count: 2,
	created_at: '2026-10-01T08:00:00Z',
	stage_changed_at: '2026-11-02T09:30:00Z',
	closed_at: '2026-11-02T09:30:00Z',
}

describe('deal stages', () => {
	it('round-trips every stage between the API and the screen', () => {
		for (const stage of STAGES) {
			expect(stageOfApi(apiOfStage(stage))).toBe(stage)
		}
		expect(apiOfStage('quote_sent')).toBe('QUOTE_SENT')
	})
})

describe('dealOfApi', () => {
	it('reads the snake_case response into the screen model', () => {
		expect(dealOfApi(API_DEAL)).toEqual({
			id: 'd1',
			customerId: 'c1',
			title: 'Fournil',
			amount: 125000,
			stage: 'lost',
			expectedCloseDate: '2026-12-15',
			ownerId: 'm1',
			source: 'salon',
			lostReason: 'Prix trop élevé',
			notes: '',
			quoteCount: 2,
			createdAt: '2026-10-01T08:00:00Z',
			stageChangedAt: '2026-11-02T09:30:00Z',
			closedAt: '2026-11-02T09:30:00Z',
		})
	})

	it('turns absent optional fields into empty values', () => {
		const open = dealOfApi({
			...API_DEAL,
			stage: 'NEW',
			expected_close_date: null,
			owner_member_id: null,
			source: null,
			lost_reason: null,
			closed_at: null,
		})

		expect(open.ownerId).toBeNull()
		expect(open.source).toBeNull()
		expect(open.expectedCloseDate).toBeNull()
		expect(open.lostReason).toBe('')
		expect(open.closedAt).toBeNull()
	})
})

describe('dealBodyOfDraft', () => {
	it('sends integer cents, trimmed text and null for an empty note', () => {
		expect(
			dealBodyOfDraft({
				customerId: 'c1',
				title: '  Fournil ',
				amount: 125000,
				expectedCloseDate: null,
				ownerId: 'm1',
				source: 'salon',
				notes: '   ',
			}),
		).toEqual({
			customer_id: 'c1',
			title: 'Fournil',
			amount_cents: 125000,
			expected_close_date: null,
			owner_member_id: 'm1',
			source: 'salon',
			notes: null,
		})
	})
})

describe('dealQuoteOfApi', () => {
	const quote: Schemas.QuoteResponse = {
		id: 'q1',
		organization_id: 'org',
		reference: null,
		title: 'Devis fournil',
		customer_id: 'c1',
		customer_context_id: 'ctx',
		deal_id: 'd1',
		status: 'SENT',
		net_cents: 100000,
		vat_breakdown: [],
		gross_cents: 120000,
		lines: [],
		created_at: '2026-10-02T08:00:00Z',
		updated_at: '2026-10-02T08:00:00Z',
	}

	it('keeps the quotes linked to a deal, with their net amount', () => {
		expect(dealQuoteOfApi(quote)).toEqual({
			id: 'q1',
			dealId: 'd1',
			number: null,
			title: 'Devis fournil',
			status: 'sent',
			subtotal: 100000,
			createdAt: '2026-10-02T08:00:00Z',
		})
	})

	it('ignores a quote that answers no deal', () => {
		expect(dealQuoteOfApi({ ...quote, deal_id: null })).toBeNull()
	})
})

describe('summaryOfApi', () => {
	it('carries the backend figures without recomputing them', () => {
		const summary = summaryOfApi({
			open_count: 3,
			open_amount_cents: 900,
			stages: [
				{ stage: 'NEW', count: 3, amount_cents: 900 },
				{ stage: 'WON', count: 1, amount_cents: 500 },
			],
			won_count: 1,
			lost_count: 2,
			won_amount_cents: 500,
			win_rate: 33,
			conversions: 4,
		})

		expect(summary.winRate).toBe(33)
		expect(summary.stages[0]).toEqual({ stage: 'new', count: 3, amount: 900 })
		expect(summary.wonAmount).toBe(500)
		expect(summary.conversions).toBe(4)
	})

	it('keeps a missing win rate empty', () => {
		const summary = summaryOfApi({
			open_count: 0,
			open_amount_cents: 0,
			stages: [],
			won_count: 0,
			lost_count: 0,
			won_amount_cents: 0,
			win_rate: null,
			conversions: 0,
		})

		expect(summary.winRate).toBeNull()
	})
})
