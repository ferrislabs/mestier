import { describe, expect, it } from 'vitest'
import { seedConversions, seedDeals } from '#/pages/deals/lib/deal-seed'
import { customer } from '#/pages/deals/lib/testing'

describe('seedDeals', () => {
	it('creates one deal per customer with a stage, mapped from the customer stage', () => {
		const customers = [
			customer({ id: 'n', pipeline_stage: 'NEW' }),
			customer({ id: 'c', pipeline_stage: 'CONTACTED' }),
			customer({ id: 'q', pipeline_stage: 'QUALIFIED' }),
			customer({ id: 's', pipeline_stage: 'QUOTE_SENT' }),
			customer({ id: 'w', pipeline_stage: 'WON', status: 'CLIENT' }),
			customer({ id: 'l', pipeline_stage: 'LOST' }),
		]
		const stages = seedDeals(customers).map((item) => [
			item.customerId,
			item.stage,
		])
		expect(stages).toEqual([
			['n', 'new'],
			['c', 'visit'],
			['q', 'visit'],
			['s', 'quote_sent'],
			['w', 'won'],
			['l', 'lost'],
		])
	})

	it('skips archived customers', () => {
		const customers = [
			customer({ id: 'b', status: 'ARCHIVED' }),
			customer({ id: 'c' }),
		]
		expect(seedDeals(customers).map((item) => item.customerId)).toEqual(['c'])
	})

	it('dates the deal from the customer and closes won and lost ones', () => {
		const [won, open] = seedDeals([
			customer({
				id: 'w',
				pipeline_stage: 'WON',
				created_at: '2026-03-01T08:00:00Z',
				updated_at: '2026-04-02T08:00:00Z',
			}),
			customer({ id: 'o' }),
		])
		expect(won).toMatchObject({
			title: 'Opportunité Famille Dupont',
			amount: 0,
			createdAt: '2026-03-01',
			stageChangedAt: '2026-04-02',
			closedAt: '2026-04-02',
		})
		expect(open?.closedAt).toBeNull()
		expect(open?.id).toBe('deal-o')
	})
})

describe('seedConversions', () => {
	it('records the customers that already won', () => {
		expect(
			seedConversions([
				customer({
					id: 'w',
					pipeline_stage: 'WON',
					status: 'CLIENT',
					updated_at: '2026-04-02T08:00:00Z',
				}),
				customer({ id: 'p' }),
			]),
		).toEqual({ w: '2026-04-02' })
	})
})
