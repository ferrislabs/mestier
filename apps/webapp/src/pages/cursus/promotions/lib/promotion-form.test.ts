import { describe, expect, it } from 'vitest'
import {
	EMPTY_PROMOTION_FORM,
	promotionFormErrors,
	toPromotionRequest,
} from '#/pages/cursus/promotions/lib/promotion-form'

describe('promotion form', () => {
	it('requires every field on an empty form', () => {
		const errors = promotionFormErrors(EMPTY_PROMOTION_FORM)
		expect(Object.keys(errors).sort()).toEqual([
			'endYear',
			'label',
			'startYear',
			'trainingId',
		])
	})

	it('rejects an end year before the start year', () => {
		const errors = promotionFormErrors({
			trainingId: 't-1',
			label: 'P1',
			startYear: '2026',
			endYear: '2025',
		})
		expect(errors.endYear).toBeDefined()
	})

	it('builds the request payload with numeric years', () => {
		expect(
			toPromotionRequest({
				trainingId: 't-1',
				label: ' CAP 2026 ',
				startYear: '2026',
				endYear: '2028',
			}),
		).toEqual({
			training_id: 't-1',
			label: 'CAP 2026',
			start_year: 2026,
			end_year: 2028,
		})
	})
})
