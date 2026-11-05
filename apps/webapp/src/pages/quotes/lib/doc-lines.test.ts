import { describe, expect, it } from 'vitest'
import { duplicateItem, moveItem } from './doc-lines'

describe('moveItem', () => {
	it('swaps with the neighbour', () => {
		expect(moveItem(['a', 'b', 'c'], 1, -1)).toEqual(['b', 'a', 'c'])
		expect(moveItem(['a', 'b', 'c'], 1, 1)).toEqual(['a', 'c', 'b'])
	})

	it('stays put at the edges', () => {
		expect(moveItem(['a', 'b'], 0, -1)).toEqual(['a', 'b'])
		expect(moveItem(['a', 'b'], 1, 1)).toEqual(['a', 'b'])
	})
})

describe('duplicateItem', () => {
	it('inserts the copy right after its source', () => {
		expect(duplicateItem(['a', 'b'], 0, (source) => `${source}2`)).toEqual([
			'a',
			'a2',
			'b',
		])
	})

	it('ignores an unknown index', () => {
		expect(duplicateItem(['a'], 3, (source) => `${source}2`)).toEqual(['a'])
	})
})
