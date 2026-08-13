import { describe, expect, it } from 'vitest'
import { badgeColorFor } from '#/pages/automation/lib/connector-badge'

describe('badgeColorFor', () => {
	it('is deterministic for the same seed', () => {
		expect(badgeColorFor('http')).toBe(badgeColorFor('http'))
	})

	it('assigns a color to a family it has never seen before', () => {
		expect(() =>
			badgeColorFor('a-brand-new-family-nobody-registered'),
		).not.toThrow()
		expect(badgeColorFor('a-brand-new-family-nobody-registered')).toMatch(
			/^bg-/,
		)
	})

	it('does not crash on an empty seed', () => {
		expect(badgeColorFor('')).toMatch(/^bg-/)
	})
})
