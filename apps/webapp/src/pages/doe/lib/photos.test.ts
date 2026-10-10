import { describe, expect, it } from 'vitest'
import { photoPageCount, photoSummary } from '#/pages/doe/lib/photos'

describe('photoPageCount', () => {
	it('rounds up to whole pages', () => {
		expect(photoPageCount(5, 'two')).toBe(3)
		expect(photoPageCount(4, 'two')).toBe(2)
		expect(photoPageCount(5, 'four')).toBe(2)
		expect(photoPageCount(3, 'one')).toBe(3)
	})

	it('is zero without photos', () => {
		expect(photoPageCount(0, 'four')).toBe(0)
	})
})

describe('photoSummary', () => {
	it('reads as a sentence', () => {
		expect(photoSummary(5, 'two')).toBe('5 photos, 2 par page = 3 pages')
		expect(photoSummary(1, 'one')).toBe('1 photo, 1 par page = 1 page')
	})
})
