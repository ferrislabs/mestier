import { describe, expect, it } from 'vitest'
import {
	amountOf,
	dealDraftOf,
	EMPTY_DEAL_FORM,
	validateDealForm,
} from '#/pages/deals/lib/deal-draft'
import { createDeal } from '#/pages/deals/lib/pipeline'
import { ctxAt, TODAY } from '#/pages/deals/lib/testing'

const prefill = { id: 'c1', ownerId: 'julien', source: 'salon' as const }

describe('amountOf', () => {
	it('reads an amount in euros and leaves zero for empty', () => {
		expect(amountOf('')).toBe(0)
		expect(amountOf('  ')).toBe(0)
		expect(amountOf('1 250,50')).toBe(125050)
		expect(amountOf('800')).toBe(80000)
		expect(amountOf('abc')).toBeNull()
		expect(amountOf('12,345')).toBeNull()
	})
})

describe('dealDraftOf', () => {
	it('takes the owner and the source of the customer', () => {
		const draft = dealDraftOf(prefill, {
			title: '  Rénovation cuisine ',
			amount: '4 200',
			expectedCloseDate: '2026-12-15',
		})
		expect(draft).toEqual({
			customerId: 'c1',
			title: 'Rénovation cuisine',
			amount: 420000,
			expectedCloseDate: '2026-12-15',
			ownerId: 'julien',
			source: 'salon',
			notes: '',
		})
	})

	it('gives a null close date for an empty date', () => {
		expect(dealDraftOf(prefill, EMPTY_DEAL_FORM).expectedCloseDate).toBeNull()
	})
})

describe('validateDealForm', () => {
	it('flags the title and the amount', () => {
		expect(
			validateDealForm(prefill, {
				title: 'Pose de volets',
				amount: '900',
				expectedCloseDate: '',
			}),
		).toEqual({})
		const errors = validateDealForm(prefill, {
			title: ' ',
			amount: 'abc',
			expectedCloseDate: '',
		})
		expect(errors.title).toBeTruthy()
		expect(errors.amount).toBeTruthy()
		expect(
			validateDealForm(prefill, {
				title: 'Pose',
				amount: '-50',
				expectedCloseDate: '',
			}).amount,
		).toBeTruthy()
	})
})

describe('createDeal', () => {
	it('creates the deal at the new stage from a valid draft', () => {
		const draft = dealDraftOf(prefill, {
			title: 'Pose de volets',
			amount: '900',
			expectedCloseDate: '',
		})
		const created = createDeal([], ctxAt(TODAY, ['d-new']), draft)
		expect(created[0]).toMatchObject({ stage: 'new', amount: 90000 })
	})
})
