import type { Deal, DealDraft, DealStage } from '#/pages/deals/lib/deal-types'

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
