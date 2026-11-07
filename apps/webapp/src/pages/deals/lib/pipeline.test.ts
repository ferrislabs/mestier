import { describe, expect, it } from 'vitest'
import {
	createDeal,
	dealsInStage,
	deleteDeal,
	moveDeal,
	pipelineSummary,
	STAGES,
	validateDealDraft,
} from '#/pages/deals/lib/pipeline'
import { ctxAt, deal, quote, TODAY } from '#/pages/deals/lib/testing'

const draft = {
	customerId: 'c8',
	title: 'Bureaux',
	amount: 500000,
	expectedCloseDate: null,
	ownerId: 'baptiste',
	source: 'salon' as const,
	notes: '',
}

describe('createDeal', () => {
	it('starts a new deal as a new request', () => {
		const created = createDeal([], ctxAt(TODAY, ['d-new']), draft)
		expect(created).toHaveLength(1)
		expect(created[0]).toMatchObject({
			id: 'd-new',
			stage: 'new',
			closedAt: null,
			createdAt: TODAY,
			stageChangedAt: TODAY,
		})
	})

	it('needs a title and a customer', () => {
		const base = [deal()]
		expect(createDeal(base, ctxAt(TODAY), { ...draft, title: ' ' })).toBe(base)
		expect(createDeal(base, ctxAt(TODAY), { ...draft, customerId: '' })).toBe(
			base,
		)
	})
})

describe('validateDealDraft', () => {
	it('flags the title, the customer and a negative or fractional amount', () => {
		expect(validateDealDraft(draft)).toEqual([])
		expect(validateDealDraft({ ...draft, title: ' ', customerId: '' })).toEqual(
			['L’intitulé est obligatoire.', 'Choisis un client.'],
		)
		expect(validateDealDraft({ ...draft, amount: -1 })).toEqual([
			'Le montant estimé doit être positif.',
		])
	})
})

describe('moveDeal', () => {
	it('winning closes the deal today', () => {
		const won = moveDeal([deal()], ctxAt(TODAY), 'd1', 'won')
		expect(won[0]).toMatchObject({
			stage: 'won',
			closedAt: TODAY,
			lostReason: '',
			stageChangedAt: TODAY,
		})
	})

	it('losing requires a reason', () => {
		const base = [deal()]
		expect(moveDeal(base, ctxAt(TODAY), 'd1', 'lost')).toBe(base)
		expect(moveDeal(base, ctxAt(TODAY), 'd1', 'lost', ' ')).toBe(base)
		const lost = moveDeal(base, ctxAt(TODAY), 'd1', 'lost', ' Prix trop élevé ')
		expect(lost[0]).toMatchObject({
			stage: 'lost',
			lostReason: 'Prix trop élevé',
			closedAt: TODAY,
		})
	})

	it('moving to an open stage reopens a closed deal', () => {
		const closed = deal({
			stage: 'lost',
			lostReason: 'Pas de réponse',
			closedAt: '2026-10-20',
		})
		const reopened = moveDeal([closed], ctxAt(TODAY), 'd1', 'negotiation')
		expect(reopened[0]).toMatchObject({
			stage: 'negotiation',
			closedAt: null,
			lostReason: '',
		})
	})

	it('does nothing for the same stage or an unknown deal', () => {
		const base = [deal()]
		expect(moveDeal(base, ctxAt(TODAY), 'd1', 'new')).toBe(base)
		expect(moveDeal(base, ctxAt(TODAY), 'nope', 'won')).toBe(base)
	})
})

describe('deleteDeal', () => {
	it('refuses a deal linked to a quote', () => {
		const base = [deal(), deal({ id: 'd2' })]
		expect(deleteDeal(base, 'd1', [quote()])).toBe(base)
		expect(deleteDeal(base, 'd2', [quote()]).map((item) => item.id)).toEqual([
			'd1',
		])
		expect(deleteDeal(base, 'nope', [])).toBe(base)
	})
})

describe('dealsInStage', () => {
	it('sorts a column by latest change, the most recent move first', () => {
		const deals = [
			deal({ id: 'a', stage: 'visit', stageChangedAt: '2026-10-01' }),
			deal({ id: 'b', stage: 'visit', stageChangedAt: '2026-10-09' }),
			deal({ id: 'c', stage: 'new' }),
			deal({ id: 'd', stage: 'visit', stageChangedAt: '2026-10-09' }),
		]
		expect(dealsInStage(deals, 'visit').map((item) => item.id)).toEqual([
			'd',
			'b',
			'a',
		])
	})
})

describe('pipelineSummary', () => {
	const deals = [
		deal({ id: 'n', stage: 'new', amount: 1000 }),
		deal({ id: 'v', stage: 'visit', amount: 2000 }),
		deal({ id: 'w1', stage: 'won', amount: 5000, closedAt: '2026-10-20' }),
		deal({ id: 'w2', stage: 'won', amount: 7000, closedAt: '2026-10-30' }),
		deal({ id: 'w3', stage: 'won', amount: 9000, closedAt: '2026-01-02' }),
		deal({ id: 'l1', stage: 'lost', amount: 4000, closedAt: '2026-10-25' }),
	]

	it('counts open deals, recent wins and the win rate', () => {
		const summary = pipelineSummary(deals, {}, TODAY)
		expect(summary.openCount).toBe(2)
		expect(summary.openAmount).toBe(3000)
		expect(summary.wonCount).toBe(2)
		expect(summary.wonAmount).toBe(12000)
		expect(summary.lostCount).toBe(1)
		expect(summary.winRate).toBe(67)
		expect(summary.stages.map((row) => row.stage)).toEqual([...STAGES])
	})

	it('has no win rate without a closed deal in the window', () => {
		expect(pipelineSummary([deal()], {}, TODAY).winRate).toBeNull()
	})

	it('counts the conversions of the window', () => {
		const summary = pipelineSummary(
			deals,
			{ c1: '2026-10-30', c2: '2026-01-02' },
			TODAY,
		)
		expect(summary.conversions).toBe(1)
	})
})
