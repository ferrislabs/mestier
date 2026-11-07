import { addDays } from '#/pages/customers/lib/format'
import type {
	Conversions,
	Deal,
	DealContext,
	DealDraft,
	DealQuote,
	DealStage,
	IsoDate,
} from '#/pages/deals/lib/deal-types'

export const OPEN_STAGES: readonly DealStage[] = [
	'new',
	'visit',
	'quote_sent',
	'negotiation',
]

export const STAGES: readonly DealStage[] = [...OPEN_STAGES, 'won', 'lost']

export const STAGE_LABELS: Record<DealStage, string> = {
	new: 'Nouvelle demande',
	visit: 'Visite et chiffrage',
	quote_sent: 'Devis envoyé',
	negotiation: 'Négociation',
	won: 'Gagné',
	lost: 'Perdu',
}

export const LOST_REASONS: readonly string[] = [
	'Prix trop élevé',
	'Délai trop long',
	'Concurrent retenu',
	'Projet abandonné',
	'Pas de réponse',
]

export const SUMMARY_WINDOW_DAYS = 90

export function isOpenStage(stage: DealStage): boolean {
	return OPEN_STAGES.includes(stage)
}

export function findDeal(
	deals: readonly Deal[],
	dealId: string | null,
): Deal | undefined {
	return deals.find((deal) => deal.id === dealId)
}

export function validateDealDraft(draft: DealDraft): string[] {
	const errors: string[] = []
	if (draft.title.trim() === '') errors.push('L’intitulé est obligatoire.')
	if (draft.customerId === '') errors.push('Choisis un client.')
	if (!Number.isInteger(draft.amount) || draft.amount < 0)
		errors.push('Le montant estimé doit être positif.')
	return errors
}

export function createDeal(
	deals: readonly Deal[],
	context: DealContext,
	draft: DealDraft,
): readonly Deal[] {
	if (validateDealDraft(draft).length > 0) return deals
	const deal: Deal = {
		...draft,
		id: context.newId(),
		title: draft.title.trim(),
		notes: draft.notes.trim(),
		stage: 'new',
		lostReason: '',
		createdAt: context.today,
		stageChangedAt: context.today,
		closedAt: null,
	}
	return [...deals, deal]
}

export function updateDeal(
	deals: readonly Deal[],
	dealId: string,
	patch: Partial<DealDraft>,
): readonly Deal[] {
	return deals.map((deal) =>
		deal.id === dealId
			? {
					...deal,
					...patch,
					title: patch.title !== undefined ? patch.title.trim() : deal.title,
				}
			: deal,
	)
}

function revise(
	deals: readonly Deal[],
	dealId: string,
	patch: Partial<Deal>,
): readonly Deal[] {
	return deals.map((deal) =>
		deal.id === dealId ? { ...deal, ...patch } : deal,
	)
}

export function moveDeal(
	deals: readonly Deal[],
	context: DealContext,
	dealId: string,
	stage: DealStage,
	lostReason = '',
): readonly Deal[] {
	const deal = findDeal(deals, dealId)
	if (!deal || deal.stage === stage) return deals
	if (stage === 'lost') {
		const reason = lostReason.trim()
		if (reason === '') return deals
		return revise(deals, dealId, {
			stage,
			lostReason: reason,
			stageChangedAt: context.today,
			closedAt: context.today,
		})
	}
	return revise(deals, dealId, {
		stage,
		lostReason: '',
		stageChangedAt: context.today,
		closedAt: stage === 'won' ? context.today : null,
	})
}

export function deleteDeal(
	deals: readonly Deal[],
	dealId: string,
	quotes: readonly DealQuote[],
): readonly Deal[] {
	const linked = quotes.some((quote) => quote.dealId === dealId)
	if (linked || !findDeal(deals, dealId)) return deals
	return deals.filter((deal) => deal.id !== dealId)
}

export function dealsInStage(deals: readonly Deal[], stage: DealStage): Deal[] {
	return deals
		.filter((deal) => deal.stage === stage)
		.reverse()
		.sort((a, b) => b.stageChangedAt.localeCompare(a.stageChangedAt))
}

export type PipelineSummary = {
	openCount: number
	openAmount: number
	stages: { stage: DealStage; count: number; amount: number }[]
	wonCount: number
	lostCount: number
	wonAmount: number
	winRate: number | null
	conversions: number
}

export function totalAmount(deals: readonly Deal[]): number {
	return deals.reduce((sum, deal) => sum + deal.amount, 0)
}

export function pipelineSummary(
	deals: readonly Deal[],
	conversions: Conversions,
	today: IsoDate,
): PipelineSummary {
	const since = addDays(today, -SUMMARY_WINDOW_DAYS)
	const stages = STAGES.map((stage) => {
		const column = deals.filter((deal) => deal.stage === stage)
		return { stage, count: column.length, amount: totalAmount(column) }
	})
	const open = stages.filter((row) => isOpenStage(row.stage))
	const closedRecently = (stage: DealStage) =>
		deals.filter(
			(deal) =>
				deal.stage === stage &&
				deal.closedAt !== null &&
				deal.closedAt >= since,
		)
	const won = closedRecently('won')
	const lost = closedRecently('lost')
	return {
		openCount: open.reduce((sum, row) => sum + row.count, 0),
		openAmount: open.reduce((sum, row) => sum + row.amount, 0),
		stages,
		wonCount: won.length,
		lostCount: lost.length,
		wonAmount: totalAmount(won),
		winRate:
			won.length + lost.length === 0
				? null
				: Math.round((won.length / (won.length + lost.length)) * 100),
		conversions: Object.values(conversions).filter((date) => date >= since)
			.length,
	}
}
