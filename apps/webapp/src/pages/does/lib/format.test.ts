import { describe, expect, it } from 'vitest'
import { formatIssuedOn } from '#/pages/does/lib/format'

describe('formatIssuedOn', () => {
	it('renders dd/mm/yyyy', () => {
		expect(formatIssuedOn('2026-09-03')).toBe('03/09/2026')
		expect(formatIssuedOn('2026-09-03T10:00:00Z')).toBe('03/09/2026')
	})
})
