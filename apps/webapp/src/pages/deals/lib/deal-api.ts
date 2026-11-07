import type { Schemas } from '#/api/api.client'
import type { ApiDeal, ApiDealStage, ApiDealSummary } from '#/hooks/use-deals'
import type {
	Deal,
	DealDraft,
	DealQuote,
	DealQuoteStatus,
	DealStage,
} from '#/pages/deals/lib/deal-types'
import type { PipelineSummary } from '#/pages/deals/lib/pipeline'

const STAGE_OF_API: Record<ApiDealStage, DealStage> = {
	NEW: 'new',
	VISIT: 'visit',
	QUOTE_SENT: 'quote_sent',
	NEGOTIATION: 'negotiation',
	WON: 'won',
	LOST: 'lost',
}

const API_OF_STAGE: Record<DealStage, ApiDealStage> = {
	new: 'NEW',
	visit: 'VISIT',
	quote_sent: 'QUOTE_SENT',
	negotiation: 'NEGOTIATION',
	won: 'WON',
	lost: 'LOST',
}

const QUOTE_STATUS_OF_API: Record<Schemas.QuoteStatus, DealQuoteStatus> = {
	DRAFT: 'draft',
	SENT: 'sent',
	ACCEPTED: 'accepted',
	DECLINED: 'declined',
	CANCELLED: 'cancelled',
	SUPERSEDED: 'superseded',
}

export function stageOfApi(stage: ApiDealStage): DealStage {
	return STAGE_OF_API[stage]
}

export function apiOfStage(stage: DealStage): ApiDealStage {
	return API_OF_STAGE[stage]
}

export function dealOfApi(deal: ApiDeal): Deal {
	return {
		id: deal.id,
		customerId: deal.customer_id,
		title: deal.title,
		amount: deal.amount_cents,
		stage: stageOfApi(deal.stage),
		expectedCloseDate: deal.expected_close_date ?? null,
		ownerId: deal.owner_member_id ?? null,
		source: deal.source ?? null,
		lostReason: deal.lost_reason ?? '',
		notes: deal.notes ?? '',
		quoteCount: deal.quote_count,
		createdAt: deal.created_at,
		stageChangedAt: deal.stage_changed_at,
		closedAt: deal.closed_at ?? null,
	}
}

export function dealBodyOfDraft(draft: DealDraft) {
	return {
		customer_id: draft.customerId,
		title: draft.title.trim(),
		amount_cents: draft.amount,
		expected_close_date: draft.expectedCloseDate,
		owner_member_id: draft.ownerId,
		source: draft.source,
		notes: draft.notes.trim() === '' ? null : draft.notes.trim(),
	}
}

export function dealQuoteOfApi(quote: Schemas.QuoteResponse): DealQuote | null {
	if (!quote.deal_id) return null
	return {
		id: quote.id,
		dealId: quote.deal_id,
		number: quote.reference ?? null,
		title: quote.title,
		status: QUOTE_STATUS_OF_API[quote.status],
		subtotal: quote.net_cents,
		createdAt: quote.created_at,
	}
}

export function summaryOfApi(summary: ApiDealSummary): PipelineSummary {
	return {
		openCount: summary.open_count,
		openAmount: summary.open_amount_cents,
		stages: summary.stages.map((row) => ({
			stage: stageOfApi(row.stage),
			count: row.count,
			amount: row.amount_cents,
		})),
		wonCount: summary.won_count,
		lostCount: summary.lost_count,
		wonAmount: summary.won_amount_cents,
		winRate: summary.win_rate ?? null,
		conversions: summary.conversions,
	}
}

export const EMPTY_SUMMARY: PipelineSummary = {
	openCount: 0,
	openAmount: 0,
	stages: [],
	wonCount: 0,
	lostCount: 0,
	wonAmount: 0,
	winRate: null,
	conversions: 0,
}
