import { describe, expect, it } from 'vitest'
import {
	dealsInStage,
	isOpenStage,
	OPEN_STAGES,
	STAGES,
	totalAmount,
	validateDealDraft,
} from '#/pages/deals/lib/pipeline'
import { deal } from '#/pages/deals/lib/testing'

const draft = {
	customerId: 'c8',
	title: 'Bureaux',
	amount: 500000,
	expectedCloseDate: null,
	ownerId: 'baptiste',
	source: 'salon' as const,
	notes: '',
}

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

describe('stages', () => {
	it('keeps four open stages before won and lost', () => {
		expect(OPEN_STAGES).toEqual(['new', 'visit', 'quote_sent', 'negotiation'])
		expect(STAGES).toEqual([...OPEN_STAGES, 'won', 'lost'])
		expect(STAGES.filter(isOpenStage)).toEqual([...OPEN_STAGES])
	})
})

describe('totalAmount', () => {
	it('sums the integer cents of a column', () => {
		expect(
			totalAmount([deal({ amount: 1000 }), deal({ id: 'd2', amount: 250 })]),
		).toBe(1250)
		expect(totalAmount([])).toBe(0)
	})
})
