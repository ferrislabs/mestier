import { describe, expect, it, vi } from 'vitest'
import {
	pickedProductIds,
	removeSuppliesBySource,
	toggleProduct,
} from '#/pages/doe/lib/picker'
import type { DoeSupply } from '#/pages/doe/types'

function supply(id: string, source: string | null): DoeSupply {
	return {
		id,
		designation: id,
		source_product_id: source,
		photos: [],
		datasheet_keys: [],
	}
}

describe('toggleProduct', () => {
	it('appends the returned draft when checked', async () => {
		const draft = supply('new', 'prod-2')
		const fetchDraft = vi.fn().mockResolvedValue(draft)
		const result = await toggleProduct(
			[supply('a', 'prod-1')],
			'prod-2',
			true,
			fetchDraft,
		)
		expect(fetchDraft).toHaveBeenCalledWith('prod-2')
		expect(result.map((entry) => entry.id)).toEqual(['a', 'new'])
	})

	it('removes the supply whose source product matches when unchecked', async () => {
		const fetchDraft = vi.fn()
		const result = await toggleProduct(
			[supply('a', 'prod-1'), supply('b', 'prod-2'), supply('c', null)],
			'prod-2',
			false,
			fetchDraft,
		)
		expect(fetchDraft).not.toHaveBeenCalled()
		expect(result.map((entry) => entry.id)).toEqual(['a', 'c'])
	})

	it('does not draft a product that is already present', async () => {
		const fetchDraft = vi.fn()
		const list = [supply('a', 'prod-1')]
		const result = await toggleProduct(list, 'prod-1', true, fetchDraft)
		expect(fetchDraft).not.toHaveBeenCalled()
		expect(result).toBe(list)
	})
})

describe('pickedProductIds', () => {
	it('collects only supplies that came from a product', () => {
		const ids = pickedProductIds([supply('a', 'p'), supply('b', null)])
		expect([...ids]).toEqual(['p'])
	})
})

describe('removeSuppliesBySource', () => {
	it('leaves manual supplies alone', () => {
		expect(removeSuppliesBySource([supply('b', null)], 'p')).toHaveLength(1)
	})
})
