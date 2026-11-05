import { describe, expect, it } from 'vitest'
import { organizationHue } from '#/modules/org-hue'

describe('organizationHue', () => {
	it('is stable for a given slug', () => {
		expect(organizationHue('menuiserie-duval')).toBe(
			organizationHue('menuiserie-duval'),
		)
	})

	it('stays within the hue wheel', () => {
		for (const slug of ['a', 'menuiserie-duval', 'atelier-demo', '']) {
			const hue = organizationHue(slug)
			expect(hue).toBeGreaterThanOrEqual(0)
			expect(hue).toBeLessThan(360)
		}
	})

	it('tells two organizations apart', () => {
		expect(organizationHue('menuiserie-duval')).not.toBe(
			organizationHue('duval-renovation'),
		)
	})
})
