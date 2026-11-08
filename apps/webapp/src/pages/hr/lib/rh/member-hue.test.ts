import { describe, expect, it } from 'vitest'
import { memberHue } from '#/pages/hr/lib/rh/member-hue'

describe('memberHue', () => {
	it('is deterministic for a given member id', () => {
		expect(memberHue('0192f0aa-1111')).toBe(memberHue('0192f0aa-1111'))
	})

	it('stays inside the 0-359 colour wheel', () => {
		for (const id of [
			'',
			'a',
			'baptiste',
			'0192f0aa-9c3e-7b55-a1d2-4f6e8b0c1d2e',
		]) {
			const hue = memberHue(id)
			expect(hue).toBeGreaterThanOrEqual(0)
			expect(hue).toBeLessThan(360)
		}
	})

	it('spreads different ids over different hues', () => {
		const hues = new Set(
			['alix', 'manon', 'jules', 'sacha', 'lou'].map((id) => memberHue(id)),
		)
		expect(hues.size).toBeGreaterThan(1)
	})
})
