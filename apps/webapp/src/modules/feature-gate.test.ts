import { describe, expect, it } from 'vitest'
import { isFeatureVisible } from '#/modules/feature-gate'

describe('isFeatureVisible', () => {
	it('shows a target that requires no feature, even while loading', () => {
		expect(isFeatureVisible({}, [], false)).toBe(true)
	})

	it('hides a gated target while the features are loading', () => {
		expect(isFeatureVisible({ requiredFeature: 'drive_ai' }, [], false)).toBe(
			false,
		)
	})

	it('hides a gated target whose feature is absent', () => {
		expect(isFeatureVisible({ requiredFeature: 'drive_ai' }, [], true)).toBe(
			false,
		)
	})

	it('shows a gated target whose feature is enabled', () => {
		expect(
			isFeatureVisible({ requiredFeature: 'drive_ai' }, ['drive_ai'], true),
		).toBe(true)
	})
})
