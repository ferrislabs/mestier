import { describe, expect, it } from 'vitest'
import { formatNewsDate, newsState, newsStateLabel } from './news-state'

const NOW = new Date('2026-11-10T12:00:00Z')

describe('newsState', () => {
	it('is a draft without a publication date', () => {
		expect(newsState(null, NOW)).toBe('draft')
		expect(newsState(undefined, NOW)).toBe('draft')
		expect(newsStateLabel('draft')).toBe('Brouillon')
	})

	it('is scheduled for a future date', () => {
		expect(newsState('2026-11-11T00:00:00Z', NOW)).toBe('scheduled')
		expect(newsStateLabel('scheduled')).toBe('Programmée')
	})

	it('is published for a past or present date', () => {
		expect(newsState('2026-11-10T12:00:00Z', NOW)).toBe('published')
		expect(newsState('2026-01-01T00:00:00Z', NOW)).toBe('published')
		expect(newsStateLabel('published')).toBe('Publiée')
	})
})

describe('formatNewsDate', () => {
	it('shows a dash without a date', () => {
		expect(formatNewsDate(null)).toBe('–')
	})

	it('formats a date in French', () => {
		expect(formatNewsDate('2026-11-10T12:00:00Z')).toMatch(/2026/)
	})
})
