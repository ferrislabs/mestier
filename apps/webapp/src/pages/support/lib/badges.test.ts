import { describe, expect, it } from 'vitest'
import { normalizeBadge, sameBadge } from '#/pages/support/lib/badges'

describe('normalizeBadge', () => {
	it('collapses inner whitespace and trims', () => {
		expect(normalizeBadge('  Sous   garantie ')).toBe('Sous garantie')
	})
})

describe('sameBadge', () => {
	it('compares labels without regard to case', () => {
		expect(sameBadge('Urgent', 'urgent')).toBe(true)
		expect(sameBadge('Urgent', 'Garantie')).toBe(false)
	})
})
