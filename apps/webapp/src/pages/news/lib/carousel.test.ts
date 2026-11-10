import { describe, expect, it } from 'vitest'
import { clampIndex, indexForKey, nextIndex, prevIndex } from './carousel'

describe('carousel', () => {
	it('clamps next and previous within bounds', () => {
		expect(nextIndex(0, 3)).toBe(1)
		expect(nextIndex(2, 3)).toBe(2)
		expect(prevIndex(2, 3)).toBe(1)
		expect(prevIndex(0, 3)).toBe(0)
	})

	it('clamps an arbitrary index', () => {
		expect(clampIndex(-4, 3)).toBe(0)
		expect(clampIndex(9, 3)).toBe(2)
		expect(clampIndex(5, 0)).toBe(0)
	})

	it('maps arrow keys to an index and ignores other keys', () => {
		expect(indexForKey('ArrowRight', 0, 3)).toBe(1)
		expect(indexForKey('ArrowLeft', 1, 3)).toBe(0)
		expect(indexForKey('ArrowLeft', 0, 3)).toBe(0)
		expect(indexForKey('Enter', 1, 3)).toBeNull()
	})
})
