import { describe, expect, it } from 'vitest'
import {
	defaultContent,
	enabledNumberedSections,
	isSectionEmpty,
	moveItem,
	newSection,
	numberSections,
	suppliesOf,
	supplyBadgeCounts,
	supplyNumber,
	upsertSupply,
	withSupplies,
} from '#/pages/doe/lib/sections'
import type { DoeSection, DoeSupply } from '#/pages/doe/types'

function section(id: string, enabled: boolean): DoeSection {
	return { id, title: id, enabled, content: defaultContent('plans') }
}

describe('numberSections', () => {
	it('skips disabled sections when numbering', () => {
		const result = numberSections([
			section('a', true),
			section('b', false),
			section('c', true),
		])
		expect(result.map((entry) => entry.number)).toEqual([1, null, 2])
	})

	it('lists only enabled sections in enabledNumberedSections', () => {
		const result = enabledNumberedSections([
			section('a', false),
			section('b', true),
		])
		expect(result).toHaveLength(1)
		expect(result[0]?.number).toBe(1)
		expect(result[0]?.section.id).toBe('b')
	})
})

describe('supplyNumber', () => {
	it('joins the section number and the one-based index', () => {
		expect(supplyNumber(2, 2)).toBe('2.3')
		expect(supplyNumber(1, 0)).toBe('1.1')
	})
})

describe('moveItem', () => {
	it('moves an item up and down', () => {
		expect(moveItem([1, 2, 3], 1, -1)).toEqual([2, 1, 3])
		expect(moveItem([1, 2, 3], 1, 1)).toEqual([1, 3, 2])
	})

	it('leaves the list untouched at the bounds', () => {
		const list = [1, 2, 3]
		expect(moveItem(list, 0, -1)).toBe(list)
		expect(moveItem(list, 2, 1)).toBe(list)
		expect(moveItem(list, 5, -1)).toBe(list)
	})
})

describe('defaultContent', () => {
	it('gives each kind its default content', () => {
		expect(defaultContent('supplies')).toEqual({
			kind: 'supplies',
			maker_label: 'manufacturer',
			supplies: [],
		})
		expect(defaultContent('photos')).toEqual({
			kind: 'photos',
			per_page: 'two',
			photos: [],
		})
		expect(defaultContent('plans')).toEqual({ kind: 'plans', plans: [] })
		expect(defaultContent('blank_pages')).toEqual({
			kind: 'blank_pages',
			count: 1,
		})
	})
})

describe('newSection', () => {
	it('is enabled, trimmed and carries its kind content', () => {
		const created = newSection('id-1', '  Notes  ', 'blank_pages')
		expect(created).toEqual({
			id: 'id-1',
			title: 'Notes',
			enabled: true,
			content: { kind: 'blank_pages', count: 1 },
		})
	})
})

describe('supplyBadgeCounts', () => {
	const base: DoeSupply = {
		id: 's',
		designation: 'Banc',
		photos: [],
		datasheet_keys: [],
	}

	it('flags a supply with neither photo nor datasheet', () => {
		expect(supplyBadgeCounts(base).isEmpty).toBe(true)
	})

	it('counts photos and datasheets', () => {
		const counts = supplyBadgeCounts({
			...base,
			photos: [{ key: 'k' }],
			datasheet_keys: ['a', 'b'],
		})
		expect(counts).toEqual({ photos: 1, datasheets: 2, isEmpty: false })
	})
})

describe('supply helpers', () => {
	const supplyA: DoeSupply = {
		id: 'a',
		designation: 'A',
		photos: [],
		datasheet_keys: [],
	}

	it('upserts by id', () => {
		expect(upsertSupply([], supplyA)).toEqual([supplyA])
		const renamed = { ...supplyA, designation: 'B' }
		expect(upsertSupply([supplyA], renamed)).toEqual([renamed])
	})

	it('reads and writes the supplies of one section', () => {
		const sections: DoeSection[] = [
			{
				id: 's1',
				title: 'Végétaux',
				enabled: true,
				content: { kind: 'supplies', maker_label: 'nursery', supplies: [] },
			},
		]
		const next = withSupplies(sections, 's1', [supplyA])
		expect(suppliesOf(next, 's1')).toEqual([supplyA])
		expect(suppliesOf(next, 'missing')).toEqual([])
	})
})

describe('isSectionEmpty', () => {
	it('lets an empty section go and keeps one holding content', () => {
		const empty = newSection('a', 'Jeux', 'supplies')
		expect(isSectionEmpty(empty)).toBe(true)
		const photos: DoeSection = {
			...newSection('b', 'Photos', 'photos'),
			content: {
				kind: 'photos',
				per_page: 'two',
				photos: [{ key: 'k', caption: null }],
			},
		}
		expect(isSectionEmpty(photos)).toBe(false)
		expect(isSectionEmpty(newSection('c', 'Notes', 'blank_pages'))).toBe(true)
	})
})
