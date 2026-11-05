import { describe, expect, it } from 'vitest'
import {
	collectColumns,
	flattenItem,
	toItems,
} from '#/pages/automation/lib/items'

describe('items', () => {
	it('flattens nested objects with dotted keys', () => {
		expect(flattenItem({ a: 1, b: { c: 'x', d: [1, 2] } })).toEqual({
			a: '1',
			'b.c': 'x',
			'b.d': '[1,2]',
		})
	})

	it('collects the columns of all rows in first-seen order', () => {
		expect(collectColumns([{ a: '1' }, { b: '2', a: '3' }])).toEqual(['a', 'b'])
	})

	it('wraps a single value as one item and an absent value as none', () => {
		expect(toItems(undefined)).toEqual([])
		expect(toItems(null)).toEqual([])
		expect(toItems({ a: 1 })).toEqual([{ a: 1 }])
		expect(toItems([1, 2])).toEqual([1, 2])
	})
})
