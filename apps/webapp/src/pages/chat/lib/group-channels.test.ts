import { describe, expect, it } from 'vitest'
import { groupChannelsByCategory } from '#/pages/chat/lib/group-channels'

function category(id: string, position: number) {
	return {
		id,
		organization_id: 'org-1',
		name: id,
		position,
		created_at: '',
		updated_at: '',
	}
}

function channel(id: string, categoryId: string | null, position: number) {
	return {
		id,
		organization_id: 'org-1',
		channel_type: 'TEXT' as const,
		name: id,
		topic: null,
		position,
		category_id: categoryId,
		parent_id: null,
		origin_message_id: null,
		archived: false,
		created_at: '',
		updated_at: '',
	}
}

describe('groupChannelsByCategory', () => {
	it('groups channels under their category, ordered by position', () => {
		const categories = [category('cat-b', 1), category('cat-a', 0)]
		const channels = [
			channel('ch-2', 'cat-a', 1),
			channel('ch-1', 'cat-a', 0),
			channel('ch-3', 'cat-b', 0),
		]

		const groups = groupChannelsByCategory(categories, channels)

		expect(groups.map((g) => g.category?.id)).toEqual(['cat-a', 'cat-b'])
		expect(groups[0]?.channels.map((c) => c.id)).toEqual(['ch-1', 'ch-2'])
		expect(groups[1]?.channels.map((c) => c.id)).toEqual(['ch-3'])
	})

	it('keeps a category with no channels', () => {
		const groups = groupChannelsByCategory([category('cat-a', 0)], [])
		expect(groups).toHaveLength(1)
		expect(groups[0]?.channels).toEqual([])
	})

	it('puts uncategorized channels in a trailing null-category group', () => {
		const channels = [channel('ch-1', null, 0)]
		const groups = groupChannelsByCategory([category('cat-a', 0)], channels)

		expect(groups.map((g) => g.category?.id ?? null)).toEqual(['cat-a', null])
		expect(groups[1]?.channels.map((c) => c.id)).toEqual(['ch-1'])
	})

	it('returns nothing for an empty organization', () => {
		expect(groupChannelsByCategory([], [])).toEqual([])
	})
})
