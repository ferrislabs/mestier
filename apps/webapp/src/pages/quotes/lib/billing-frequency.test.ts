import { describe, expect, it } from 'vitest'
import {
	BILLING_FREQUENCIES,
	FREQUENCY_LABELS,
	FREQUENCY_SUFFIX,
	isBillingFrequency,
} from './billing-frequency'

describe('billing frequency', () => {
	it('offers one-shot, monthly, quarterly and yearly billing', () => {
		expect(BILLING_FREQUENCIES).toEqual([
			'one_shot',
			'monthly',
			'quarterly',
			'yearly',
		])
		expect(FREQUENCY_LABELS.one_shot).toBe('Ponctuel')
		expect(FREQUENCY_LABELS.quarterly).toBe('Trimestriel')
	})

	it('prints no suffix for a one-shot line', () => {
		expect(FREQUENCY_SUFFIX.one_shot).toBe('')
		expect(FREQUENCY_SUFFIX.monthly).toBe('/ mois')
		expect(FREQUENCY_SUFFIX.yearly).toBe('/ an')
	})

	it('recognises a frequency', () => {
		expect(isBillingFrequency('monthly')).toBe(true)
		expect(isBillingFrequency('weekly')).toBe(false)
		expect(isBillingFrequency(3)).toBe(false)
	})
})
