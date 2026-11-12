import { describe, expect, it } from 'vitest'
import { countOf } from '#/pages/cursus/nav/lib/count-of'

describe('countOf', () => {
	it('is unknown while the list loads', () => {
		expect(countOf(undefined)).toBeUndefined()
	})

	it('prefers the paginated total over the page length', () => {
		expect(countOf({ data: [1], pagination: { total: 58 } })).toBe(58)
	})

	it('falls back to the rows of an unpaginated list', () => {
		expect(countOf({ data: [1, 2] })).toBe(2)
	})
})
