import { describe, expect, it } from 'vitest'
import { formatDate } from '#/pages/cursus/shared/lib/format'

describe('formatDate', () => {
	it('formats an ISO date as dd/MM/yyyy', () => {
		expect(formatDate('2026-09-01')).toBe('01/09/2026')
	})

	it('keeps only the date part of a timestamp', () => {
		expect(formatDate('2026-09-01T10:00:00Z')).toBe('01/09/2026')
	})

	it('renders a dash when absent', () => {
		expect(formatDate(null)).toBe('—')
	})
})
