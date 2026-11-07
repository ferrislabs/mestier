import { describe, expect, it } from 'vitest'
import {
	asItem,
	collectColumns,
	flattenItem,
	outputItemCount,
	outputItems,
	parseStepOutput,
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

describe('asItem', () => {
	it('keeps an object and wraps anything else under « valeur »', () => {
		expect(asItem({ a: 1 })).toEqual({ a: 1 })
		expect(asItem(5)).toEqual({ valeur: 5 })
		expect(asItem('x')).toEqual({ valeur: 'x' })
		expect(asItem(null)).toEqual({ valeur: null })
		expect(asItem([1])).toEqual({ valeur: [1] })
	})
})

describe('parseStepOutput', () => {
	it('reads a list as the items of a node without branches', () => {
		expect(parseStepOutput([{ a: 1 }, { a: 2 }])).toEqual({
			kind: 'main',
			items: [{ a: 1 }, { a: 2 }],
		})
	})

	it('reads one list per branch for a condition, in the order of the branches', () => {
		expect(parseStepOutput({ Else: [{ a: 2 }], Then: [{ a: 1 }] })).toEqual({
			kind: 'branched',
			lists: [
				{ branch: 'Then', items: [{ a: 1 }] },
				{ branch: 'Else', items: [{ a: 2 }] },
			],
		})
	})

	it('reads one list per branch for a loop and wraps its raw values', () => {
		expect(parseStepOutput({ Each: [1, { n: 2 }], After: [] })).toEqual({
			kind: 'branched',
			lists: [
				{ branch: 'Each', items: [{ valeur: 1 }, { n: 2 }] },
				{ branch: 'After', items: [] },
			],
		})
	})

	it('reads a value of an older run, which is not a list, as one item', () => {
		expect(parseStepOutput({ matched: true })).toEqual({
			kind: 'main',
			items: [{ matched: true }],
		})
		expect(parseStepOutput({ items: [1] })).toEqual({
			kind: 'main',
			items: [{ items: [1] }],
		})
		expect(parseStepOutput('texte')).toEqual({
			kind: 'main',
			items: [{ valeur: 'texte' }],
		})
	})

	it('reads an absent output as no item', () => {
		expect(parseStepOutput(undefined)).toEqual({ kind: 'main', items: [] })
		expect(parseStepOutput(null)).toEqual({ kind: 'main', items: [] })
	})

	it('does not take an object that mixes branches and other keys for a branched output', () => {
		expect(parseStepOutput({ Then: [], matched: true })).toEqual({
			kind: 'main',
			items: [{ Then: [], matched: true }],
		})
	})
})

describe('outputItems', () => {
	const branched = { Then: [{ n: 1 }], Else: [{ n: 2 }, { n: 3 }] }

	it('gives the items of a node without branches whatever the branch asked for', () => {
		expect(outputItems([{ a: 1 }], [])).toEqual([{ a: 1 }])
		expect(outputItems([{ a: 1 }], ['Then'])).toEqual([{ a: 1 }])
	})

	it('gives the items of the branches that feed the reader', () => {
		expect(outputItems(branched, ['Else'])).toEqual([{ n: 2 }, { n: 3 }])
		expect(outputItems(branched, ['Then'])).toEqual([{ n: 1 }])
	})

	it('gives every branch when none is named', () => {
		expect(outputItems(branched, [])).toEqual([{ n: 1 }, { n: 2 }, { n: 3 }])
	})
})

describe('outputItemCount', () => {
	it('counts the items of a list and the sum of the branches', () => {
		expect(outputItemCount([{ a: 1 }, { a: 2 }])).toBe(2)
		expect(outputItemCount({ Then: [{}], Else: [{}, {}] })).toBe(3)
		expect(outputItemCount(undefined)).toBe(0)
	})
})
